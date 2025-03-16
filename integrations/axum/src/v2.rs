use crate::extractors::TCtxFunc;
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Multipart, State},
    http::{HeaderValue, Method, Response, StatusCode, request::Parts},
    response::{IntoResponse, Sse, sse::Event},
    routing::{MethodFilter, on, post},
};
use futures::{
    FutureExt, SinkExt, Stream, StreamExt, TryStreamExt, channel::oneshot, pin_mut,
    stream::FuturesUnordered,
};
use rspc_procedure::{Procedure, ProcedureError, ProcedureStream, Procedures, ResolverError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    convert::Infallible,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    time::Duration,
};
use streamunordered::{StreamUnordered, StreamYield};

thread_local! {
    static FLUSHED: RefCell<Option<oneshot::Sender<()>>> = RefCell::new(None);
}

pub fn flush() {
    FLUSHED.take().map(|c| c.send(()));
}

macro_rules! rspc_err_json {
	($json:expr) => {
		json!({ "__rspc": $json })
	}
}

macro_rules! rspc_err_body {
    ($json:expr) => {
        Body::from(
            serde_json::to_vec(&rspc_err_json!($json))
                .expect("converting known json should never fail"),
        )
    };
}

pub struct Endpoint<TCtx, TCtxFn, S> {
    procedures: Procedures<TCtx>,
    ctx_fn: TCtxFn,
    manual_stream_flushing: bool,
    phantom: PhantomData<S>,
}

impl<TCtx, TCtxFn, TCtxFnMarker, S> Endpoint<TCtx, TCtxFn, (TCtxFnMarker, S)>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, S, TCtxFnMarker>,
{
    pub fn new(procedures: impl Borrow<Procedures<TCtx>>, ctx_fn: TCtxFn) -> Self {
        Self {
            procedures: procedures.borrow().clone(),
            ctx_fn,
            manual_stream_flushing: false,
            phantom: PhantomData,
        }
    }

    pub fn manual_stream_flushing(mut self) -> Self {
        self.manual_stream_flushing = true;
        self
    }

    pub fn build(self) -> Router<S> {
        let procedures = Arc::new(self.procedures);

        Router::<S>::new()
            .route(
                "/{id}",
                on(MethodFilter::GET.or(MethodFilter::POST), {
                    let procedures = procedures.clone();
                    let ctx_fn = self.ctx_fn.clone();

                    move |state: State<S>, req: axum::extract::Request<Body>| {
                        let procedures = procedures.clone();

                        async move {
                            let (parts, body) = req.into_parts();

                            let procedure_name = parts.uri.path()[1..].to_string();

                            let Some(procedure) =
                                procedures.get(&Cow::Borrowed(procedure_name.as_str()))
                            else {
                                return Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .header("Content-Type", "application/json")
                                    .body(rspc_err_body!(format!(
                                        "procedure not found: {procedure_name}"
                                    )))
                                    .unwrap();
                            };

                            match parts.method {
                                Method::GET => handle_procedure(
                                    ctx_fn,
                                    parts
                                        .uri
                                        .query()
                                        .map(|query| form_urlencoded::parse(query.as_bytes()))
                                        .and_then(|mut params| {
                                            params.find(|e| e.0 == "input").map(|e| e.1)
                                        })
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
                                            .then(|| {
                                                serde_json::from_slice(body.to_vec().as_slice())
                                            })
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
                    }
                }),
            )
            .route("/", {
                post(
                    move |state: State<S>, mut req: axum::extract::Request<Body>| {
                        let procedures = procedures.clone();

                        async move {
                            let (parts, body) = req.into_parts();
                            // let Ok(parts) = req.extract_parts::<Parts>().await;

                            // let multipart = match req.extract::<Multipart, _>().await {
                            //     Ok(m) => m,
                            //     Err(_) => {
                            //         return Response::builder()
                            //             .status(StatusCode::BAD_REQUEST)
                            //             .header("Content-Type", "application/json")
                            //             .body(rspc_err_body!("invalid multipart data"))
                            //             .unwrap();
                            //     }
                            // };

                            #[derive(Deserialize)]
                            struct BatchInput(Vec<(String, Value)>);

                            let Ok(input) = ({
                                let body = to_bytes(body, usize::MAX).await.unwrap(); // TODO: error handling
                                serde_json::from_slice::<BatchInput>(body.to_vec().as_slice())
                            }) else {
                                return Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .header("Content-Type", "application/json")
                                    .body(rspc_err_body!("invalid batch body"))
                                    .unwrap();
                            };

                            let input = input
                                .0
                                .into_iter()
                                .map(|(procedure_name, input)| {
                                    let Some(procedure) = procedures.get(procedure_name.as_str())
                                    else {
                                        return Err(Response::builder()
                                            .status(StatusCode::NOT_FOUND)
                                            .header("Content-Type", "application/json")
                                            .body(rspc_err_body!(format!(
                                                "procedure '{procedure_name}' not found"
                                            )))
                                            .unwrap());
                                    };

                                    Ok((procedure, Some(input)))
                                })
                                .collect::<Result<Vec<_>, _>>();

                            handle_batch(
                                self.ctx_fn,
                                match input {
                                    Ok(v) => v,
                                    Err(e) => return e,
                                },
                                parts,
                                state.0,
                                self.manual_stream_flushing,
                            )
                            .await
                        }
                    },
                )
            })
    }
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

        Sse::new(futures::stream::unfold(Some(stream), async |stream| {
            let mut stream = stream?;

            let Some(v) = next(&mut stream).await else {
                return Some((Ok(Event::default().data("stopped")), None));
            };

            let data = v.map_or_else(
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
            );

            let is_error = matches!(data, SSEEvent::Error { .. });

            Some((
                Ok::<_, Infallible>(Event::default().json_data(data).unwrap()),
                (!is_error).then_some(stream),
            ))
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

async fn handle_batch<TCtx, TCtxFn, TCtxFnMarker, TState>(
    ctx_fn: TCtxFn,
    inputs: Vec<(&Procedure<TCtx>, Option<Value>)>,
    req_parts: Parts,
    state: TState,
    manual_stream_flushing: bool,
) -> Response<Body>
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
    TState: Send + Sync + 'static,
{
    let mut stream = StreamUnordered::new();

    let flushes = FuturesUnordered::new();

    let stream_response =
        req_parts.headers.get("rspc-batch-mode") == Some(&HeaderValue::from_static("stream"));

    for (procedure, input) in inputs.into_iter() {
        let ctx = match ctx_fn.exec(req_parts.clone(), &state).await {
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

        let mut procedure_stream =
            procedure.exec_with_deserializer(ctx, input.unwrap_or(Value::Null));

        let mut flush_tx_opt = if manual_stream_flushing && stream_response {
            let (tx, rx) = futures::channel::oneshot::channel();
            flushes.push(rx);
            Some(tx)
        } else {
            None
        };

        stream.insert(futures::stream::once(async move {
            let next_fut = next(&mut procedure_stream);
            pin_mut!(next_fut);

            match futures::future::poll_fn(|cx| {
                if let Some(flush_tx) = flush_tx_opt.take() {
                    FLUSHED.set(Some(flush_tx));

                    let res = next_fut.poll_unpin(cx);

                    flush_tx_opt = FLUSHED.take();

                    res
                } else {
                    next_fut.poll_unpin(cx)
                }
            })
            .await
            {
                Some(Ok(value)) => (200, value),
                Some(Err(NextError::Procedure(status, body))) => {
                    (status.as_u16(), rspc_err_json!(body))
                }
                Some(Err(NextError::Resolver(resolver_error))) => {
                    (500, json!(resolver_error.value()))
                }
                None => (500, rspc_err_json!("procedure didn't produce a value")),
                _ => unreachable!(),
            }
        }));
    }

    if stream_response {
        let stream = futures::stream::unfold(stream, |mut stream| async move {
            let Some((item, i)) = stream.next().await else {
                return None;
            };

            let out = match item {
                StreamYield::Item(item) => {
                    let stream_index = i - 1;

                    format!(
                        "{stream_index}:{}\n",
                        serde_json::to_string(&json!(item))
                            .expect("failed to stringify serde_json::Value")
                    )
                }
                StreamYield::Finished(s) => {
                    s.remove(Pin::new(&mut stream));

                    String::new()
                }
            };

            Some((Ok::<_, Infallible>(out), stream))
        });

        let body = if flushes.is_empty() {
            Body::from_stream(stream)
        } else {
            let (mut tx, rx) = futures::channel::mpsc::channel(1);

            tokio::spawn(async move {
                pin_mut!(stream);
                while let Some(item) = stream.next().await {
                    if let Err(_) = tx.send(item).await {
                        return;
                    }
                }
            });

            futures::future::join_all(flushes).await;

            Body::from_stream(rx.into_stream())
        };

        Response::builder()
            .header("Transfer-Encoding", "chunked")
            .body(body)
            .unwrap()
    } else {
        let mut responses = vec![None; stream.len()];

        while let Some((item, i)) = stream.next().await {
            let stream_index = i - 1;

            match item {
                StreamYield::Item(item) => {
                    responses[stream_index].get_or_insert(item);
                }
                StreamYield::Finished(s) => {
                    s.remove(Pin::new(&mut stream));
                }
            };
        }

        Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&json!(responses)).unwrap()))
            .unwrap()
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

    impl<'a, TCtx, TState, TCtxFnMarker, TCtxFn> Executor<'a, TCtx, TState, TCtxFnMarker, TCtxFn>
    where
        TCtx: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TState, TCtxFnMarker>,
        TState: Send + Sync + 'static,
    {
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
        let (parts, body) = Executor::new(&procedure, || ()).execute().await;
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
            .execute()
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
    async fn stream_resolver_error() {
        let procedure = Procedure::<()>::new(|_, _| {
            ProcedureStream::from_stream(futures::stream::iter([
                Ok(1),
                Err(ProcedureError::Resolver(ResolverError::new(
                    json!("error"),
                    None::<Infallible>,
                ))),
                Ok(3),
            ]))
        });

        let (parts, body) = Executor::new(&procedure, || ())
            .modify_request(|b| b.header("Accept", "text/event-stream"))
            .execute()
            .await;
        let events = assert_sse(&parts, body);

        assert_eq!(
            events,
            vec![
                json!({ "item": 1 }),
                json!({ "error": { "status": 500, "data": "error" }}),
            ]
        );
    }

    #[tokio::test]
    async fn invalid_input_400() {
        let procedure = Procedure::new(|_: (), _| ProcedureStream::from_future(async { Ok(()) }));

        let (parts, body) = Executor::new(&procedure, || ())
            .with_input(serde_json::from_slice(&[]))
            .execute()
            .await;
        let body = assert_json(&parts, body);

        assert_rspc_err(&parts, &body, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn failed_ctx_500() {
        let procedure = Procedure::new(|_: (), _| ProcedureStream::from_future(async { Ok(()) }));

        let (parts, body) = Executor::new(&procedure, |_: tower_cookies::Cookies| ())
            .execute()
            .await;
        let body = assert_json(&parts, body);

        assert_rspc_err(&parts, &body, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn no_value_500() {
        let procedure = Procedure::new(|_: (), _| {
            ProcedureStream::from_stream(futures::stream::empty::<Result<(), _>>())
        });

        let (parts, body) = Executor::new(&procedure, || ()).execute().await;
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

        let (parts, body) = Executor::new(&procedure, || ()).execute().await;
        let body = assert_json(&parts, body);

        assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            body,
            json!(CustomError {
                message: "error".to_string()
            })
        );
    }

    #[tokio::test]
    async fn batch_query() {
        handle_batch(ctx_fn, inputs, req_parts, state, manual_stream_flushing)
    }
}
