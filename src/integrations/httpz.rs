use futures::{SinkExt, StreamExt};
use httpz::{
    cookie::CookieJar,
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    ConcreteRequest, Endpoint, EndpointResult, GenericEndpoint, HttpEndpoint, QueryParms,
};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    internal::{
        jsonrpc::{self, RequestInner, ResponseInner},
        Procedure, ProcedureKind, RequestContext,
    },
    Router,
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
        (&Method::GET, _) => {
            handle_http(ctx_fn(), ProcedureKind::Query, req, router.queries()).await
        }
        (&Method::POST, _) => {
            handle_http(ctx_fn(), ProcedureKind::Mutation, req, router.mutations()).await
        }
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

pub async fn handle_http<TCtx>(
    ctx: TCtx,
    kind: ProcedureKind,
    req: ConcreteRequest,
    procedures: &BTreeMap<String, Procedure<TCtx>>,
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

    todo!();
}

pub async fn handle_json_rpc<TCtx>(
    ctx: TCtx,
    req: jsonrpc::Request,
    operations: &BTreeMap<String, Procedure<TCtx>>,
) -> Result<jsonrpc::Response, ()>
where
    TCtx: 'static,
{
    if !req.jsonrpc.is_none() && req.jsonrpc.as_deref() != Some("2.0") {
        // return Err(ExecError::InvalidJsonRpcVersion);
        todo!();
    }

    // TODO: Remove duplication
    match req.inner {
        RequestInner::Query { path, input } => {
            let y = operations
                .get(&path)
                // .ok_or(ExecError::OperationNotFound(path))?
                .unwrap()
                // .unwrap()
                .exec
                .call(
                    ctx,
                    input.unwrap_or(Value::Null),
                    RequestContext {
                        kind: ProcedureKind::Query,
                        path,
                    },
                )
                .unwrap()
                .into_value()
                .await
                .unwrap();

            Ok(jsonrpc::Response {
                jsonrpc: "2.0",
                id: req.id,
                inner: ResponseInner::Ok { result: y },
            })
        }
        RequestInner::Mutation { path, input } => {
            let y = operations
                .get(&path)
                // .ok_or(ExecError::OperationNotFound(path))?
                .unwrap()
                // .unwrap()
                .exec
                .call(
                    ctx,
                    input.unwrap_or(Value::Null),
                    RequestContext {
                        kind: ProcedureKind::Mutation,
                        path,
                    },
                )
                .unwrap()
                .into_value()
                .await
                .unwrap();

            Ok(jsonrpc::Response {
                jsonrpc: "2.0",
                id: req.id,
                inner: ResponseInner::Ok { result: y },
            })
        }
        RequestInner::Subscription { path, input } => {
            // let y = operations
            //     .get(&path)
            //     // .ok_or(ExecError::OperationNotFound(path))?
            //     .unwrap()
            //     // .unwrap()
            //     .exec
            //     .call(
            //         ctx,
            //         input.unwrap_or(Value::Null),
            //         RequestContext {
            //             kind: ProcedureKind::Mutation,
            //             path,
            //         },
            //     )
            //     .unwrap()
            //     .into_value()
            //     .await
            //     .unwrap();

            todo!();
        }
        RequestInner::StopSubscription => {
            todo!();
        }
    }
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
        // let subscriptions = HashMap::new();
        let (tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

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
                                    println!("{:?}", text);
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
                            // #[cfg(feature = "tracing")]
                            // tracing::error!("Error in websocket: {}", err);

                            todo!();
                        },
                        None => return,
                    };

                    socket.send(Message::Text(serde_json::to_string(&handle_json_rpc(ctx.clone(), req, router.subscriptions()).await.unwrap()).unwrap())).await.unwrap();
                }
            }
        }
    })
}
