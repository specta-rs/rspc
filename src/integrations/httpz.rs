use futures::{Future, SinkExt, StreamExt};
use httpz::{
    axum::axum::extract::{FromRequest, Path, RequestParts},
    http::{self, Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    ConcreteRequest, Endpoint, EndpointResult, GenericEndpoint, HttpEndpoint, QueryParms,
};
use serde_json::Value;
use std::{any::Any, collections::HashMap, marker::PhantomData, pin::Pin, sync::Arc};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::{
    internal::{
        jsonrpc::{self, RequestId, RequestInner, ResponseInner},
        ProcedureKind, RequestContext, ValueOrStream,
    },
    ExecError, Router,
};

pub use httpz::cookie::CookieJar;

// TODO: This request extractor system needs a huge refactor!!!!
// TODO: Can we avoid needing to box the extractors????
// TODO: Support for up to 16 extractors
// TODO: Debug bounds on `::Rejection` should only happen in the `tracing` feature is enabled
// TODO: Allow async context functions

pub enum TCtxFuncResult<'a, TCtx> {
    Value(Result<TCtx, ExecError>),
    Future(Pin<Box<dyn Future<Output = Result<TCtx, ExecError>> + Send + 'a>>),
}

pub trait TCtxFunc<TCtx, TMarker>: Clone + Send + Sync + 'static
where
    TCtx: Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx>;
}

pub struct NoArgMarker(PhantomData<()>);
impl<TCtx, TFunc> TCtxFunc<TCtx, NoArgMarker> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce() -> TCtx + Clone + Send + Sync + 'static,
{
    fn exec<'a>(&self, _request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        TCtxFuncResult::Value(Ok(self.clone()()))
    }
}

pub struct OneArgAxumRequestMarker<T1>(PhantomData<T1>);
impl<T1, TCtx, TFunc> TCtxFunc<TCtx, OneArgAxumRequestMarker<T1>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1))
        }))
    }
}

pub struct TwoArgAxumRequestMarker<T1, T2>(PhantomData<(T1, T2)>);
impl<T1, T2, TCtx, TFunc> TCtxFunc<TCtx, TwoArgAxumRequestMarker<T1, T2>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2))
        }))
    }
}

pub struct ThreeArgAxumRequestMarker<T1, T2, T3>(PhantomData<(T1, T2, T3)>);
impl<T1, T2, T3, TCtx, TFunc> TCtxFunc<TCtx, ThreeArgAxumRequestMarker<T1, T2, T3>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3))
        }))
    }
}

pub struct FourArgAxumRequestMarker<T1, T2, T3, T4>(PhantomData<(T1, T2, T3, T4)>);
impl<T1, T2, T3, T4, TCtx, TFunc> TCtxFunc<TCtx, FourArgAxumRequestMarker<T1, T2, T3, T4>> for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3, T4) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
    <T4 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T4: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t4 = T4::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 4: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3, t4))
        }))
    }
}

pub struct FiveArgAxumRequestMarker<T1, T2, T3, T4, T5>(PhantomData<(T1, T2, T3, T4, T5)>);
impl<T1, T2, T3, T4, T5, TCtx, TFunc> TCtxFunc<TCtx, FiveArgAxumRequestMarker<T1, T2, T3, T4, T5>>
    for TFunc
where
    TCtx: Send + Sync + 'static,
    TFunc: FnOnce(T1, T2, T3, T4, T5) -> TCtx + Clone + Send + Sync + 'static,
    <T1 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T1: FromRequest<Vec<u8>> + Send + 'static,
    <T2 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T2: FromRequest<Vec<u8>> + Send + 'static,
    <T3 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T3: FromRequest<Vec<u8>> + Send + 'static,
    <T4 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T4: FromRequest<Vec<u8>> + Send + 'static,
    <T5 as FromRequest<Vec<u8>>>::Rejection: std::fmt::Debug,
    T5: FromRequest<Vec<u8>> + Send + 'static,
{
    fn exec<'a>(&self, request: &'a mut RequestParts<Vec<u8>>) -> TCtxFuncResult<'a, TCtx> {
        let this = self.clone();
        TCtxFuncResult::Future(Box::pin(async move {
            let t1 = T1::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t2 = T2::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 2: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t3 = T3::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 3: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t4 = T4::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 4: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            let t5 = T5::from_request(request).await.map_err(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("error executing axum extractor 5: {:?}", _err);

                ExecError::AxumExtractorError
            })?;

            Ok(this(t1, t2, t3, t4, t5))
        }))
    }
}

struct Ctx<TCtxFn, TCtx, TMeta, TCtxFnMarker>
where
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
{
    router: Arc<Router<TCtx, TMeta>>,
    ctx_fn: TCtxFn,
    phantom: PhantomData<TCtxFnMarker>,
}

// Rust's #[derive(Clone)] would require `Clone` on all the generics even though that isn't strictly required.
impl<TCtxFn, TCtx, TMeta, TCtxFnMarker> Clone for Ctx<TCtxFn, TCtx, TMeta, TCtxFnMarker>
where
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            ctx_fn: self.ctx_fn.clone(),
            phantom: PhantomData,
        }
    }
}

// TODO: Move this into httpz
pub fn clone_req(req: &RequestParts<Vec<u8>>) -> RequestParts<Vec<u8>> {
    RequestParts::new(
        http::Request::builder()
            .method(req.method().clone())
            .uri(req.uri().clone())
            .version(req.version().clone())
            .body(req.body().unwrap().clone())
            .unwrap(),
    )
}

async fn handler<'a, TCtxFn, TCtx, TMeta, TCtxFnMarker>(
    Ctx { router, ctx_fn, .. }: Ctx<TCtxFn, TCtx, TMeta, TCtxFnMarker>,
    req: ConcreteRequest,
    cookies: CookieJar,
) -> EndpointResult
where
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
{
    let mut req = RequestParts::new(req);

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rspc/ws") => handle_websocket(ctx_fn, req, cookies, router),
        // // TODO: `/jsonrpc` compatible endpoint for both GET and POST & maybe websocket?
        (&Method::GET, _) => handle_http(ctx_fn, ProcedureKind::Query, req, cookies, &router).await,
        (&Method::POST, _) => {
            handle_http(ctx_fn, ProcedureKind::Mutation, req, cookies, &router).await
        }
        _ => unreachable!(),
    }
}

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
{
    pub fn endpoint<TCtxFnMarker: Send + Sync + 'static, TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>>(
        self: Arc<Self>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        GenericEndpoint::new(
            Ctx {
                router: self,
                ctx_fn,
                phantom: PhantomData,
            },
            [Method::GET, Method::POST],
            handler,
        )
    }
}

pub async fn handle_http<TCtx, TMeta, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    kind: ProcedureKind,
    mut req: RequestParts<Vec<u8>>,
    cookies: CookieJar,
    router: &Arc<Router<TCtx, TMeta>>,
) -> EndpointResult
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let uri = req.uri().clone();
    let key = match uri.path().strip_prefix("/rspc/") {
        Some(key) => key,
        None => {
            return Ok((
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            )); // TODO: Include error information in response
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
            .unwrap()
            .is_empty()
            .then(|| serde_json::from_slice(&req.body().unwrap()))
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

            return Ok((
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            ));
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
    let cookies = Arc::new(Mutex::new(cookies)); // TODO: Avoid arcing in the future -> Allow ctx to how refs.
    handle_json_rpc(
        match ctx_fn.exec(&mut req) {
            TCtxFuncResult::Value(v) => v.unwrap(),
            TCtxFuncResult::Future(v) => v.await.unwrap(),
        },
        // (
        //     &http::Request::builder()
        //         .method(req.method().clone())
        //         .uri(req.uri().clone())
        //         .version(req.version().clone())
        //         .body(req.body().unwrap().clone())
        //         .unwrap(),
        //     cookies.clone(),
        // ),
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
        Sender::Response(Some(resp)) => Ok((
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_vec(&resp).unwrap())
                .unwrap(),
            (*cookies.lock().await).clone(),
        )),
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

pub fn handle_websocket<TCtx, TMeta, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: RequestParts<Vec<u8>>,
    cookies: CookieJar,
    router: Arc<Router<TCtx, TMeta>>,
) -> EndpointResult
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    #[cfg(feature = "tracing")]
    tracing::debug!("Accepting websocket connection");

    let mut req2 = clone_req(&req);
    let cookies2 = cookies.clone();

    // TODO: Cookies are read only for websocket connections. This should be enforced in the public API?
    let cookies = Arc::new(Mutex::new(cookies));
    WebsocketUpgrade::from_req(
        req.try_into_request().unwrap(),
        cookies2,
        move |mut socket| async move {
            let mut subscriptions = HashMap::new();
            let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

            loop {
                tokio::select! {
                    biased; // Note: Order is important here
                    msg = rx.recv() => {
                        socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await.unwrap();
                    }
                    msg = socket.next() => {
                        let request = match msg {
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

                        handle_json_rpc(match ctx_fn.exec(&mut req2) {
                            TCtxFuncResult::Value(v) => v.unwrap(),
                            TCtxFuncResult::Future(v) => v.await.unwrap(),
                        }, request, &router, &mut Sender::Channel((&mut tx, &mut subscriptions))).await;
                    }
                }
            }
        },
    )
}
