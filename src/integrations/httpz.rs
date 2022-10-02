use futures::{Future, SinkExt, StreamExt};
use httpz::{
    axum::axum::extract::{FromRequest, RequestParts},
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    ConcreteRequest, Endpoint, EndpointResult, GenericEndpoint, HttpEndpoint, QueryParms,
};
use serde_json::Value;
use std::{collections::HashMap, marker::PhantomData, pin::Pin, sync::Arc};
use tokio::sync::mpsc;

use crate::{
    internal::{
        jsonrpc::{self, handle_json_rpc, RequestId, Sender, SubscriptionMap},
        ProcedureKind,
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
    url_prefix: Option<&'static str>,
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
            url_prefix: self.url_prefix,
            ctx_fn: self.ctx_fn.clone(),
            phantom: PhantomData,
        }
    }
}

async fn handler<'a, TCtxFn, TCtx, TMeta, TCtxFnMarker>(
    Ctx {
        router,
        url_prefix,
        ctx_fn,
        ..
    }: Ctx<TCtxFn, TCtx, TMeta, TCtxFnMarker>,
    req: ConcreteRequest,
    cookies: CookieJar,
) -> EndpointResult
where
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
{
    let websocket_url = format!("{}/ws", url_prefix.unwrap_or("/rspc"));
    match (req.method(), req.uri().path()) {
        (&Method::GET, url) if url == websocket_url => {
            handle_websocket(ctx_fn, req, cookies, router)
        }
        (&Method::GET, _) => {
            handle_http(
                ctx_fn,
                &format!("{}/", url_prefix.unwrap_or("/rspc")),
                ProcedureKind::Query,
                req,
                cookies,
                &router,
            )
            .await
        }
        (&Method::POST, _) => {
            handle_http(
                ctx_fn,
                &format!("{}/", url_prefix.unwrap_or("/rspc")),
                ProcedureKind::Mutation,
                req,
                cookies,
                &router,
            )
            .await
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
                url_prefix: None,
                ctx_fn,
                phantom: PhantomData,
            },
            [Method::GET, Method::POST],
            handler,
        )
    }

    pub fn endpoint_with_prefix<
        TCtxFnMarker: Send + Sync + 'static,
        TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
    >(
        self: Arc<Self>,
        url_prefix: &'static str,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        GenericEndpoint::new(
            Ctx {
                router: self,
                url_prefix: Some(url_prefix),
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
    url_prefix: &str,
    kind: ProcedureKind,
    req: ConcreteRequest,
    cookies: CookieJar,
    router: &Arc<Router<TCtx, TMeta>>,
) -> EndpointResult
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let uri = req.uri().clone();

    let key = match uri.path().strip_prefix(url_prefix) {
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
            .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => (!req.body().is_empty())
            .then(|| serde_json::from_slice(req.body()))
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
    let ctx = match ctx_fn.exec(&mut RequestParts::new(req)) {
        TCtxFuncResult::Value(v) => v,
        TCtxFuncResult::Future(v) => v.await,
    };
    let ctx = match ctx {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok((
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            ));
        }
    };

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
                ProcedureKind::Subscription => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Attempted to execute a subscription operation with HTTP");

                    return Ok((
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("Content-Type", "application/json")
                            .body(b"[]".to_vec())?,
                        cookies,
                    ));
                }
            },
        },
        router,
        &mut resp,
        &mut SubscriptionMap::None,
    )
    .await;

    match resp {
        Sender::Response(Some(resp)) => Ok((
            match serde_json::to_vec(&resp) {
                Ok(v) => Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(v)?,
                Err(_err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error serializing response: {}", _err);

                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "application/json")
                        .body(b"[]".to_vec())?
                }
            },
            cookies,
        )),
        _ => unreachable!(),
    }
}

pub fn handle_websocket<TCtx, TMeta, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: ConcreteRequest,
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

    WebsocketUpgrade::from_req(req, cookies, move |req, mut socket| async move {
        let mut subscriptions = HashMap::new();
        let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);
        let mut req = RequestParts::new(req);

        loop {
            tokio::select! {
                biased; // Note: Order is important here
                msg = rx.recv() => {
                    match socket.send(Message::Text(match serde_json::to_string(&msg) {
                        Ok(v) => v,
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error serializing websocket message: {}", _err);

                            continue;
                        }
                    })).await {
                        Ok(_) => {}
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error sending websocket message: {}", _err);

                            continue;
                        }
                    }
                }
                msg = socket.next() => {
                    let request = match msg {
                        Some(Ok(msg) )=> {
                           let res = match msg {
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
                            };

                            match res {
                                Ok(v) => v,
                                Err(_err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("Error parsing websocket message: {}", _err);

                                    continue;
                                }
                            }
                        }
                        Some(Err(_err)) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error in websocket: {}", _err);

                            continue;
                        },
                        None => {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Shutting down websocket connection");

                            return;
                        },
                    };

                    let ctx = match ctx_fn.exec(&mut req) {
                        TCtxFuncResult::Value(v) => v,
                        TCtxFuncResult::Future(v) => v.await,
                    };

                    handle_json_rpc(match ctx {
                        Ok(v) => v,
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error executing context function: {}", _err);

                            continue;
                        }
                    }, request, &router, &mut Sender::Channel(&mut tx),
                    &mut SubscriptionMap::Ref(&mut subscriptions)).await;
                }
            }
        }
    })
}
