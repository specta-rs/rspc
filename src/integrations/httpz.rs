use futures::{SinkExt, StreamExt};
use httpz::{
    cookie::CookieJar,
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    ConcreteRequest, Endpoint, EndpointResult, GenericEndpoint, HttpEndpoint, QueryParms,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot};

use crate::{
    internal::{
        jsonrpc::{self, RequestId, RequestInner, ResponseInner},
        ProcedureKind, RequestContext, ValueOrStream,
    },
    ExecError, Router,
};

struct Ctx<TCtxFn, TCtx, TMeta>
where
    TCtxFn: Fn() -> TCtx + Clone + Send + Sync + 'static,
    TCtx: Send + Sync + Clone + 'static, // This 'Clone' is needed for websockets
    TMeta: Send + Sync + 'static,
{
    router: Arc<Router<TCtx, TMeta>>,
    ctx_fn: TCtxFn,
}

// Rust's #[derive(Clone)] would require `Clone` on all the generics even though that isn't strictly required.
impl<TCtxFn, TCtx, TMeta> Clone for Ctx<TCtxFn, TCtx, TMeta>
where
    TCtxFn: Fn() -> TCtx + Clone + Send + Sync + 'static,
    TCtx: Send + Sync + Clone + 'static, // This 'Clone' is needed for websockets
    TMeta: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            ctx_fn: self.ctx_fn.clone(),
        }
    }
}

async fn handler<'a, TCtxFn, TCtx, TMeta>(
    Ctx { router, ctx_fn }: Ctx<TCtxFn, TCtx, TMeta>,
    req: ConcreteRequest,
    _: &'a mut CookieJar,
) -> EndpointResult
where
    TCtxFn: Fn() -> TCtx + Clone + Send + Sync + 'static,
    TCtx: Send + Sync + Clone + 'static, // This 'Clone' is needed for websockets
    TMeta: Send + Sync + 'static,
{
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rspc/ws") => handle_websocket(ctx_fn(), req, router),
        // TODO: `/jsonrpc` compatible endpoint for both GET and POST & maybe websocket?
        (&Method::GET, _) => handle_http(ctx_fn(), ProcedureKind::Query, req, &router).await,
        (&Method::POST, _) => handle_http(ctx_fn(), ProcedureKind::Mutation, req, &router).await,
        _ => unreachable!(),
    }
}

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: Send + Sync + Clone + 'static, // This 'Clone' is needed for websockets
    TMeta: Send + Sync + 'static,
{
    pub fn endpoint<TCtxFn: Fn() -> TCtx + Clone + Send + Sync + 'static>(
        self: Arc<Self>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        GenericEndpoint::new(
            Ctx {
                router: self,
                ctx_fn,
            },
            [Method::GET, Method::POST],
            handler,
        )
    }
}

pub async fn handle_http<TCtx, TMeta>(
    ctx: TCtx,
    kind: ProcedureKind,
    req: ConcreteRequest,
    router: &Arc<Router<TCtx, TMeta>>,
) -> Result<Response<Vec<u8>>, httpz::Error> {
    let key = match req.uri().path().strip_prefix("/rspc/") {
        Some(key) => key,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(b"[]".to_vec())?); // TODO: Include error information in response
        }
    };

    let input = match *req.method() {
        Method::GET => req
            .uri()
            .query_pairs()
            .map(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .flatten()
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => req
            .body()
            .is_empty()
            .then(|| serde_json::from_slice(&req.body()))
            .unwrap_or(Ok(None)),
        _ => unreachable!(),
    };

    let input = match input {
        Ok(input) => input,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!(
                "Error passing parameters to operation '{}' with key '{:?}': {}",
                kind.to_str(),
                key,
                _err
            );

            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(b"[]".to_vec())?);
        }
    };

    #[cfg(feature = "tracing")]
    tracing::debug!(
        "Executing operation '{}' with key '{}' with params {:?}",
        kind.to_str(),
        key,
        input
    );

    let mut resp = Sender::Response(None);
    handle_json_rpc(
        ctx,
        jsonrpc::Request {
            jsonrpc: None,
            id: RequestId::Null,
            inner: match kind {
                ProcedureKind::Query => jsonrpc::RequestInner::Query {
                    path: key.to_string(),
                    input,
                },
                ProcedureKind::Mutation => jsonrpc::RequestInner::Mutation {
                    path: key.to_string(),
                    input,
                },
                ProcedureKind::Subscription => todo!(),
            },
        },
        &router,
        &mut resp,
    )
    .await;

    match resp {
        Sender::Response(Some(resp)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&resp).unwrap())
            .unwrap()),
        Sender::Response(None) => todo!(),
        _ => unreachable!(),
    }
}

pub enum Sender<'a> {
    Channel(
        (
            &'a mut mpsc::Sender<jsonrpc::Response>,
            &'a mut HashMap<RequestId, oneshot::Sender<()>>,
        ),
    ),
    Response(Option<jsonrpc::Response>),
}

impl<'a> Sender<'a> {
    pub async fn send(
        &mut self,
        resp: jsonrpc::Response,
    ) -> Result<(), mpsc::error::SendError<jsonrpc::Response>> {
        match self {
            Sender::Channel((tx, _)) => tx.send(resp).await?,
            Sender::Response(o) => *o = Some(resp),
        }

        Ok(())
    }
}

pub async fn handle_json_rpc<TCtx, TMeta>(
    ctx: TCtx,
    req: jsonrpc::Request,
    router: &Arc<Router<TCtx, TMeta>>,
    tx: &mut Sender<'_>,
) where
    TCtx: 'static,
{
    if !req.jsonrpc.is_none() && req.jsonrpc.as_deref() != Some("2.0") {
        tx.send(jsonrpc::Response {
            jsonrpc: "2.0",
            id: req.id.clone(),
            result: ResponseInner::Error(ExecError::InvalidJsonRpcVersion.into()),
        })
        .await
        .unwrap();
    }

    let (path, input, procedures, sub_id) = match req.inner {
        RequestInner::Query { path, input } => (path, input, router.queries(), None),
        RequestInner::Mutation { path, input } => (path, input, router.mutations(), None),
        RequestInner::Subscription { path, input } => {
            (path, input.1, router.subscriptions(), Some(input.0))
        }
        RequestInner::SubscriptionStop { input } => {
            match tx {
                Sender::Channel((_, subscriptions)) => {
                    subscriptions.remove(&input);
                }
                Sender::Response(_) => {}
            }

            return;
        }
    };

    let result = match procedures
        .get(&path)
        .ok_or_else(|| ExecError::OperationNotFound(path.clone()))
        .and_then(|v| {
            v.exec.call(
                ctx,
                input.unwrap_or(Value::Null),
                RequestContext {
                    kind: ProcedureKind::Query,
                    path,
                },
            )
        }) {
        Ok(op) => match op.into_value_or_stream().await {
            Ok(ValueOrStream::Value(v)) => ResponseInner::Response(v),
            Ok(ValueOrStream::Stream(mut stream)) => {
                let (tx, subscriptions) = match tx {
                    Sender::Channel((tx, subscriptions)) => (tx.clone(), subscriptions),
                    Sender::Response(_) => {
                        todo!();
                    }
                };

                let id = sub_id.unwrap();
                if matches!(id, RequestId::Null) {
                    todo!();
                } else if subscriptions.contains_key(&id) {
                    todo!();
                }

                let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
                subscriptions.insert(id.clone(), shutdown_tx);
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            biased; // Note: Order matters
                            _ = &mut shutdown_rx => {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("Removing subscription with id '{:?}'", id);
                                break;
                            }
                            v = stream.next() => {
                                match v {
                                    Some(v) => {
                                        tx.send(jsonrpc::Response {
                                            jsonrpc: "2.0",
                                            id: id.clone(),
                                            result: ResponseInner::Event(v.unwrap()),
                                        })
                                        .await
                                        .unwrap();
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });

                return;
            }
            Err(err) => ResponseInner::Error(err.into()),
        },
        Err(err) => ResponseInner::Error(err.into()),
    };

    tx.send(jsonrpc::Response {
        jsonrpc: "2.0",
        id: req.id,
        result,
    })
    .await
    .unwrap();
}

pub fn handle_websocket<TCtx, TMeta>(
    ctx: TCtx,
    req: ConcreteRequest,
    router: Arc<Router<TCtx, TMeta>>,
) -> Result<Response<Vec<u8>>, httpz::Error>
where
    TCtx: Send + Sync + Clone + 'static,
    TMeta: Send + Sync + 'static,
{
    #[cfg(feature = "tracing")]
    tracing::debug!("Accepting websocket connection");

    WebsocketUpgrade::from_req(req, move |mut socket| async move {
        let mut subscriptions = HashMap::new();
        let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

        loop {
            tokio::select! {
                biased; // Note: Order is important here
                msg = rx.recv() => {
                    socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.unwrap();
                }
                msg = socket.next() => {
                    let req = match msg {
                        Some(Ok(msg) )=> {
                            match msg {
                                Message::Text(text) => {
                                    serde_json::from_str::<jsonrpc::Request>(&text)
                                }
                                Message::Binary(binary) => {
                                    serde_json::from_slice::<jsonrpc::Request>(&binary)
                                }
                                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                                    continue;
                                }
                                Message::Frame(_) => unreachable!(),
                            }
                            .unwrap() // TODO: Error handling
                        }
                        Some(Err(err)) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error in websocket: {}", err);

                            todo!();
                        },
                        None => return,
                    };

                    handle_json_rpc(ctx.clone(), req, &router, &mut Sender::Channel((&mut tx, &mut subscriptions))).await;
                }
            }
        }
    })
}
