use std::{
    borrow::{Borrow, Cow},
    convert::Infallible,
    time::Duration,
};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{request::Parts, Method, Response, StatusCode},
    response::{sse::Event, IntoResponse, Sse},
    routing::{on, MethodFilter},
    Router,
};
use rspc_procedure::Procedures;
use serde_json::{json, Value};

use crate::{
    extractors::TCtxFunc,
    jsonrpc::{self},
    jsonrpc_exec::next,
};

macro_rules! rspc_err_body {
    ($json:expr) => {
        Body::from(
            serde_json::to_vec(&json!({"__rspc": $json}))
                .expect("converting known json should never fail"),
        )
    };
}

pub fn endpoint<TCtx, TCtxFnMarker, TCtxFn, S>(
    procedures: impl Borrow<Procedures<TCtx>>,
    ctx_fn: TCtxFn,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, S, TCtxFnMarker>,
{
    let procedures = procedures.borrow().clone();

    Router::<S>::new().route(
        "/{id}",
        on(
            MethodFilter::GET.or(MethodFilter::POST),
            move |state: State<S>, req: axum::extract::Request<Body>| {
                let procedures = procedures.clone();

                async move {
                    let (parts, body) = req.into_parts();

                    match parts.method {
                        Method::GET => handle(
                            ctx_fn,
                            parts
                                .uri
                                .query()
                                .map(|query| form_urlencoded::parse(query.as_bytes()))
                                .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
                                .map(|v| serde_json::from_str(&v))
                                .unwrap_or(Ok(None as Option<Value>)),
                            parts,
                            &procedures,
                            state.0,
                        )
                        .await
                        .into_response(),
                        Method::POST => handle(
                            ctx_fn,
                            {
                                let body = to_bytes(body, usize::MAX).await.unwrap(); // TODO: error handling
                                (!body.is_empty())
                                    .then(|| serde_json::from_slice(body.to_vec().as_slice()))
                                    .unwrap_or(Ok(None))
                            },
                            parts,
                            &procedures,
                            state.0,
                        )
                        .await
                        .into_response(),
                        _ => Response::builder()
                            .status(StatusCode::METHOD_NOT_ALLOWED)
                            .header("Content-Type", "application/json")
                            .body(rspc_err_body!("only GET or POST methods are allowed"))
                            .unwrap(),
                    }
                }
            },
        ),
    )
}

async fn handle<TCtx, TCtxFn, TCtxFnMarker, TState>(
    ctx_fn: TCtxFn,
    input: Result<Option<Value>, serde_json::Error>,
    req_parts: Parts,
    procedures: &Procedures<TCtx>,
    state: TState,
) -> Response<Body>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
    TState: Send + Sync + 'static,
{
    let procedure_name = req_parts.uri.path()[1..].to_string();

    let input = match input {
        Ok(input) => input,
        Err(_err) => {
            // #[cfg(feature = "tracing")]
            // tracing::error!("Error passing parameters to operation '{procedure_name}': {_err}");

            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(rspc_err_body!("invalid input"))
                .unwrap();
        }
    };

    // #[cfg(feature = "tracing")]
    // tracing::debug!("Executing operation '{procedure_name}' with params {input:?}");

    let is_event_stream = req_parts
        .headers
        .get("Accept")
        .clone()
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "text/event-stream")
        .unwrap_or_default();

    let ctx = match ctx_fn.exec(req_parts, &state).await {
        Ok(ctx) => ctx,
        Err(_err) => {
            // #[cfg(feature = "tracing")]
            // tracing::error!("Error executing context function: {}", _err);

            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(rspc_err_body!("failed to execute context function"))
                .unwrap();
        }
    };

    let Some(procedure) = procedures.get(&Cow::Borrowed(procedure_name.as_str())) else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(rspc_err_body!("procedure not found"))
            .unwrap();
    };

    let mut stream = procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

    if is_event_stream {
        Sse::new(futures::stream::unfold(Some(stream), |stream| async move {
            let mut stream = stream?;
            Some(match next(&mut stream).await {
                Some(v) => (
                    Ok::<_, Infallible>(
                        Event::default()
                            .json_data(v.map_or_else(
                                jsonrpc::ResponseInner::Error,
                                jsonrpc::ResponseInner::Event,
                            ))
                            .unwrap(),
                    ),
                    Some(stream),
                ),
                None => (Ok(Event::default().data("stopped")), None),
            })
        }))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(5))
                .text("keep-alive"),
        )
        .into_response()
    } else {
        let first_value = next(&mut stream).await;

        match first_value {
            Some(value) => match value {
                Ok(value) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&value).expect("failed to convert json to bytes"),
                    ))
                    .unwrap(),
                Err(e) => Response::builder()
                    .status(e.code as u16)
                    .header("Content-Type", "application/json")
                    .body(rspc_err_body!(e.message))
                    .unwrap(),
            },
            None => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(rspc_err_body!("procedure didn't produce a value"))
                .unwrap(),
        }
    }
}
