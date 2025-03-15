use std::{
    borrow::{Borrow, Cow},
    convert::Infallible,
    future::{poll_fn, Future},
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
use futures::pin_mut;
use rspc_procedure::{Procedure, ProcedureError, ProcedureStream, Procedures, ResolverError};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    extractors::TCtxFunc,
    jsonrpc::{self, JsonRPCError},
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

                    let procedure_name = parts.uri.path()[1..].to_string();

                    let Some(procedure) = procedures.get(&Cow::Borrowed(procedure_name.as_str()))
                    else {
                        return Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "application/json")
                            .body(rspc_err_body!("procedure not found"))
                            .unwrap();
                    };

                    match parts.method {
                        Method::GET => handle_procedure(
                            ctx_fn,
                            parts
                                .uri
                                .query()
                                .map(|query| form_urlencoded::parse(query.as_bytes()))
                                .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
                                .map(|v| serde_json::from_str(&v))
                                .unwrap_or(Ok(None as Option<Value>)),
                            parts,
                            &procedure,
                            state.0,
                        )
                        .await
                        .into_response(),
                        Method::POST => handle_procedure(
                            ctx_fn,
                            {
                                let body = to_bytes(body, usize::MAX).await.unwrap(); // TODO: error handling
                                (!body.is_empty())
                                    .then(|| serde_json::from_slice(body.to_vec().as_slice()))
                                    .unwrap_or(Ok(None))
                            },
                            parts,
                            &procedure,
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

async fn handle_procedure<TCtx, TCtxFn, TCtxFnMarker, TState>(
    ctx_fn: TCtxFn,
    input: Result<Option<Value>, serde_json::Error>,
    req_parts: Parts,
    procedure: &Procedure<TCtx>,
    state: TState,
) -> Response<Body>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
    TState: Send + Sync + 'static,
{
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

    let mut stream = procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

    if is_event_stream {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        enum SSEEvent {
            Item(Value),
            Error { status: u16, data: Value },
        }

        Sse::new(futures::stream::unfold(Some(stream), |stream| async move {
            let mut stream = stream?;
            Some(match next(&mut stream).await {
                Some(v) => (
                    Ok::<_, Infallible>(
                        Event::default()
                            .json_data(v.map_or_else(
                                |e| match e {
                                    NextError::Procedure(code, message) => SSEEvent::Error {
                                        status: code.as_u16(),
                                        data: json!({"__rspc": message}),
                                    },
                                    NextError::Resolver(data) => SSEEvent::Error {
                                        status: 500,
                                        data: json!(data.value()),
                                    },
                                },
                                SSEEvent::Item,
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
                Err(NextError::Procedure(status, body)) => Response::builder()
                    .status(status)
                    .header("Content-Type", "application/json")
                    .body(rspc_err_body!(body))
                    .unwrap(),
                Err(NextError::Resolver(resolver_error)) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&json!(resolver_error.value())).unwrap(),
                    ))
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

enum NextError {
    Procedure(StatusCode, String),
    Resolver(ResolverError),
}

async fn next(stream: &mut ProcedureStream) -> Option<Result<serde_json::Value, NextError>> {
    stream.next().await.map(|v| {
        v.map_err(|err| match err {
            ProcedureError::NotFound => unimplemented!(), // Isn't created by this executor
            ProcedureError::Deserialize(_) => NextError::Procedure(
                StatusCode::BAD_REQUEST,
                "error deserializing procedure arguments".to_string(),
            ),
            ProcedureError::Downcast(_) => unimplemented!(), // Isn't supported by this executor
            ProcedureError::Resolver(resolver_err) => NextError::Resolver(resolver_err),
            ProcedureError::Unwind(err) => panic!("{err:?}"), // Restore previous behavior lol
                                                              // ProcedureError::Serializer(err) => panic!("{err:?}"),
        })
        .and_then(|v| {
            Ok(v.as_serialize()
                .unwrap()
                .serialize(serde_json::value::Serializer)
                .expect("Error serialzing value")) // This panicking is bad but this is the old exectuor
        })
    })
}

#[cfg(test)]
mod test {

    use std::future::IntoFuture;

    use axum::{
        body::Bytes,
        http::{self, HeaderValue},
    };
    use futures::StreamExt;
    use rspc_procedure::{Procedure, ProcedureStream};

    use super::*;

    struct Executor<'a, TCtx, TState, TCtxFnMarker, TCtxFn> {
        procedure: &'a Procedure<TCtx>,
        ctx_fn: TCtxFn,
        state: TState,
        phantom: std::marker::PhantomData<(TState, TCtxFnMarker)>,
        input: Result<Option<Value>, serde_json::Error>,
        request: http::request::Builder,
    }

    impl<'a, TCtx, TCtxFnMarker, TCtxFn> Executor<'a, TCtx, (), TCtxFnMarker, TCtxFn> {
        pub fn new(
            procedure: &'a Procedure<TCtx>,
            ctx_fn: TCtxFn,
        ) -> Executor<'a, TCtx, (), TCtxFnMarker, TCtxFn> {
            Executor {
                procedure,
                ctx_fn,
                phantom: Default::default(),
                state: (),
                input: Ok(None),
                request: http::request::Builder::new(),
            }
        }
    }

    impl<'a, TCtx, TState, TCtxFnMarker, TCtxFn> Executor<'a, TCtx, TState, TCtxFnMarker, TCtxFn> {
        pub fn with_input(mut self, input: Result<Option<Value>, serde_json::Error>) -> Self {
            self.input = input;
            self
        }

        pub fn modify_request(
            mut self,
            request: impl FnOnce(http::request::Builder) -> http::request::Builder,
        ) -> Self {
            self.request = request(self.request);
            self
        }
    }

    impl<'a, TCtx, TState, TCtxFnMarker, TCtxFn> Executor<'a, TCtx, TState, TCtxFnMarker, TCtxFn>
    where
        TCtx: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
        TState: Send + Sync + 'static,
    {
        async fn execute(self) -> (http::response::Parts, Bytes) {
            let (parts, body) = handle_procedure::<TCtx, _, _, TState>(
                self.ctx_fn,
                self.input,
                self.request.body(Body::empty()).unwrap().into_parts().0,
                self.procedure,
                self.state,
            )
            .await
            .into_parts();
            let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
            (parts, bytes)
        }
    }

    impl<'a, TCtx, TState, TCtxFnMarker, TCtxFn> IntoFuture
        for Executor<'a, TCtx, TState, TCtxFnMarker, TCtxFn>
    where
        TCtxFnMarker: Send + 'static,
        TCtx: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
        TState: Send + Sync + 'static,
    {
        type Output = (http::response::Parts, Bytes);
        type IntoFuture = futures::future::BoxFuture<'a, Self::Output>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(self.execute())
        }
    }

    fn assert_json(parts: &http::response::Parts, body: Bytes) -> Value {
        assert_eq!(
            parts.headers.get("Content-Type"),
            Some(&HeaderValue::from_str("application/json").unwrap())
        );
        serde_json::from_slice(&body).unwrap()
    }

    fn assert_sse(parts: &http::response::Parts, body: Bytes) -> Vec<Value> {
        assert_eq!(
            parts.headers.get("Content-Type"),
            Some(&HeaderValue::from_str("text/event-stream").unwrap())
        );
        std::str::from_utf8(&body)
            .unwrap()
            .split("\n\n")
            .filter_map(|s| {
                if s.starts_with("data: ") {
                    let data = &s["data: ".len()..];
                    if data == "stopped" {
                        None
                    } else {
                        Some(serde_json::from_str(data).unwrap())
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<Value>>()
    }

    fn assert_rspc_err(parts: &http::response::Parts, body: &Value, status: StatusCode) {
        assert_eq!(parts.status, status);
        assert!(body["__rspc"].is_string());
    }

    #[tokio::test]
    async fn query_200() {
        let procedure =
            Procedure::new(|_: (), _| ProcedureStream::from_future(async { Ok("Value") }));
        let (parts, body) = Executor::new(&procedure, || ()).await;
        let body = assert_json(&parts, body);

        assert_eq!(body, json!("Value"))
    }

    #[tokio::test]
    async fn stream_200() {
        let procedure = Procedure::<()>::new(|_, _| {
            ProcedureStream::from_stream(futures::stream::iter([1, 2, 3]).map(|v| Ok(v)))
        });

        let (parts, body) = Executor::new(&procedure, || ())
            .modify_request(|b| b.header("Accept", "text/event-stream"))
            .await;
        let events = assert_sse(&parts, body);

        assert_eq!(
            events,
            vec![
                json!({ "item": 1 }),
                json!({ "item": 2 }),
                json!({ "item": 3 }),
            ]
        )
    }

    #[tokio::test]
    async fn invalid_input_400() {
        let procedure = Procedure::new(|_: (), _| ProcedureStream::from_future(async { Ok(()) }));

        let (parts, body) = Executor::new(&procedure, || ())
            .with_input(serde_json::from_slice(&[]))
            .await;
        let body = assert_json(&parts, body);

        assert_rspc_err(&parts, &body, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn failed_ctx_500() {
        let procedure = Procedure::new(|_: (), _| ProcedureStream::from_future(async { Ok(()) }));

        let (parts, body) = Executor::new(&procedure, |_: tower_cookies::Cookies| ()).await;
        let body = assert_json(&parts, body);

        assert_rspc_err(&parts, &body, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn no_value_500() {
        let procedure = Procedure::new(|_: (), _| {
            ProcedureStream::from_stream(futures::stream::empty::<Result<(), _>>())
        });

        let (parts, body) = Executor::new(&procedure, || ()).await;
        let body = assert_json(&parts, body);

        assert_rspc_err(&parts, &body, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn typesafe_error_500() {
        #[derive(Serialize)]
        struct CustomError {
            message: String,
        }

        let procedure = Procedure::new(|_: (), _| {
            ProcedureStream::from_future(async {
                Err::<(), _>(ProcedureError::Resolver(ResolverError::new(
                    CustomError {
                        message: "error".to_string(),
                    },
                    None::<Infallible>,
                )))
            })
        });

        let (parts, body) = Executor::new(&procedure, || ()).await;
        let body = assert_json(&parts, body);

        assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            body,
            json!(CustomError {
                message: "error".to_string()
            })
        );
    }
}
