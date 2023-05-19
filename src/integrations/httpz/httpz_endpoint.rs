use futures::{stream::SplitSink, SinkExt, StreamExt};
use httpz::{
    http::{Method, Response, StatusCode},
    ws::{Message, Websocket, WebsocketUpgrade},
    Endpoint, GenericEndpoint, HttpEndpoint, HttpResponse,
};
use serde_json::Value;
use std::{
    borrow::Cow,
    future::{ready, Ready},
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    internal::exec::{
        self, AsyncRuntime, Executor, ExecutorResult, NoOpSubscriptionManager, SubscriptionManager,
        SubscriptionMap, TokioRuntime,
    },
    CompiledRouter,
};

use super::*;

// TODO: Make this whole file runtime agnostic once httpz is
// TODO: Remove all panics lol
// TODO: Cleanup the code and use more chaining

impl<TCtx> CompiledRouter<TCtx>
where
    TCtx: Clone + Send + Sync + 'static,
{
    pub fn endpoint<TCtxFnMarker: Send + Sync + 'static, TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>>(
        self: Arc<Self>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        let executor = Executor::new(self);

        // TODO: This should be able to call `ctn_fn` prior to the async boundary to avoid cloning it!
        // TODO: Basically httpz would need to be able to return `Response | Future<Response>` basically how rspc executor works.

        GenericEndpoint::new(
            "/:id", // TODO: I think this is Axum specific. Fix in `httpz`!
            [Method::GET, Method::POST],
            move |req: httpz::Request| {
                // TODO: It would be nice if these clones weren't per request.
                // TODO: Maybe httpz can `Box::leak` a ref to a context type and allow it to be shared.
                let executor = executor.clone();
                let ctx_fn = ctx_fn.clone();

                async move {
                    match (req.method(), &req.uri().path()[1..]) {
                        (&Method::GET, "ws") => {
                            handle_websocket(executor, ctx_fn, req).into_response()
                        }
                        (&Method::GET, _) => {
                            handle_http(executor, ctx_fn, req).await.into_response()
                        }
                        (&Method::POST, "_batch") => handle_http_batch(executor, ctx_fn, req)
                            .await
                            .into_response(),
                        (&Method::POST, _) => {
                            handle_http(executor, ctx_fn, req).await.into_response()
                        }
                        _ => todo!(),
                    }
                }
            },
        )
    }
}

#[allow(clippy::unwrap_used)] // TODO: Remove all panics lol
async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    executor: Executor<TCtx, TokioRuntime>,
    ctx_fn: TCtxFn,
    req: httpz::Request,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let path = Cow::Owned(
        match req.server() {
            #[cfg(feature = "vercel")]
            httpz::Server::Vercel => req
                .query_pairs()
                .and_then(|mut pairs| pairs.find(|e| e.0 == "rspc"))
                .map(|(_, v)| v.to_string()),
            _ => Some(req.uri().path()[1..].to_string()), // Has to be allocated because `TCtxFn` takes ownership of `req`
        }
        .unwrap(),
    );

    let cookies = req.cookies();
    let request = match *req.method() {
        Method::GET => {
            let input = req
                .query_pairs()
                .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
                .map(|v| serde_json::from_str(&v))
                .unwrap_or(Ok(None as Option<Value>))
                .unwrap();

            exec::Request::Query { path, input }
        }
        Method::POST => {
            let input = (!req.body().is_empty())
                .then(|| serde_json::from_slice(req.body()))
                .unwrap_or(Ok(None))
                .unwrap();

            exec::Request::Mutation { path, input }
        }
        _ => todo!(),
    };

    let cookie_jar = Arc::new(Mutex::new(cookies));
    let old_cookies = req.cookies().clone();

    let ctx = match ctx_fn.exec(req, Some(CookieJar::new(cookie_jar.clone()))) {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok((
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                // TODO: Props just return `None` here so that we don't allocate or need a clone.
                old_cookies, // If cookies were set in the context function they will be lost but it errored so thats probs fine.
            ));
        }
    };

    let response = match executor.execute(ctx, request, None as Option<NoOpSubscriptionManager>) {
        ExecutorResult::FutureResponse(fut) => fut.await,
        ExecutorResult::Response(response) => response,
        ExecutorResult::None => unreachable!(
            "Executor will only return none for a 'stopSubscription' event which is impossible here"
        ),
    };

    let cookies = {
        match Arc::try_unwrap(cookie_jar) {
            Ok(cookies) => cookies.into_inner().unwrap(),
            Err(cookie_jar) => {
                #[cfg(all(feature = "tracing", feature = "warning", debug_assertions))]
                tracing::warn!("Your application continued to hold a reference to the `CookieJar` after returning from your resolver. This forced rspc to clone it, but this most likely indicates a potential bug in your system.");
                #[cfg(all(not(feature = "tracing"), feature = "warning", debug_assertions))]
                println("Your application continued to hold a reference to the `CookieJar` after returning from your resolver. This forced rspc to clone it, but this most likely indicates a potential bug in your system.");

                cookie_jar.lock().unwrap().clone()
            }
        }
    };

    let resp = match serde_json::to_vec(&response) {
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
    };

    Ok((resp, cookies))
}

#[allow(clippy::unwrap_used)] // TODO: Remove this
async fn handle_http_batch<TCtx, TCtxFn, TCtxFnMarker>(
    executor: Executor<TCtx, TokioRuntime>,
    ctx_fn: TCtxFn,
    req: httpz::Request,
) -> impl HttpResponse
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let cookies = req.cookies();
    match serde_json::from_slice::<Vec<exec::Request>>(req.body()) {
        Ok(requests) => {
            let cookie_jar = Arc::new(Mutex::new(cookies));
            let old_cookies = req.cookies().clone();

            let ctx = match ctx_fn.exec(req, Some(CookieJar::new(cookie_jar.clone()))) {
                Ok(v) => v,
                Err(_err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error executing context function: {}", _err);

                    return Ok((
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("Content-Type", "application/json")
                            .body(b"[]".to_vec())?,
                        // TODO: Props just return `None` here so that we don't allocate or need a clone.
                        old_cookies, // If cookies were set in the context function they will be lost but it errored so thats probs fine.
                    ));
                }
            };

            let responses = executor
                .execute_batch(
                    ctx.clone(),
                    requests,
                    None as Option<NoOpSubscriptionManager>,
                )
                .await;

            let cookies = {
                match Arc::try_unwrap(cookie_jar) {
                    Ok(cookies) => cookies.into_inner().unwrap(),
                    Err(cookie_jar) => {
                        #[cfg(all(feature = "tracing", feature = "warning", debug_assertions))]
                        tracing::warn!("Your application continued to hold a reference to the `CookieJar` after returning from your resolver. This forced rspc to clone it, but this most likely indicates a potential bug in your system.");
                        #[cfg(all(not(feature = "tracing"), feature = "warning", debug_assertions))]
                        println("Your application continued to hold a reference to the `CookieJar` after returning from your resolver. This forced rspc to clone it, but this most likely indicates a potential bug in your system.");

                        cookie_jar.lock().unwrap().clone()
                    }
                }
            };

            match serde_json::to_vec(&responses) {
                Ok(v) => Ok((
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(v)?,
                    cookies,
                )),
                Err(_err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error serializing batch request: {}", _err);

                    Ok((
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header("Content-Type", "application/json")
                            .body(b"[]".to_vec())?,
                        cookies,
                    ))
                }
            }
        }
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error deserializing batch request: {}", _err);

            println!("Error deserializing batch request: {}", _err); // TODO

            Ok((
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(b"[]".to_vec())?,
                cookies,
            ))
        }
    }
}

pub struct WebsocketSubscriptionManager<R: AsyncRuntime>(
    Arc<Mutex<SubscriptionMap<R>>>,
    // TODO: Remove locking from this??
    // TODO: Don't use tokio so we are runtime agnostic
    Arc<tokio::sync::Mutex<SplitSink<Box<dyn Websocket + Send>, Message>>>,
);

impl<R: AsyncRuntime> Clone for WebsocketSubscriptionManager<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<R: AsyncRuntime> SubscriptionManager<R> for WebsocketSubscriptionManager<R> {
    type Map<'a> = MutexGuard<'a, SubscriptionMap<R>>;
    type SendFut<'a> = Ready<()>; // TODO: We don't use this cause of the `MutexLock` but should fix that at some point.

    fn subscriptions(&mut self) -> Self::Map<'_> {
        self.0.lock().unwrap()
    }

    fn send(&mut self, resp: exec::Response) -> Self::SendFut<'_> {
        match serde_json::to_string(&resp) {
            Ok(v) => {
                let m = self.1.clone();
                R::spawn(async move {
                    m.lock().await.send(Message::Text(v)).await.unwrap();
                });
            }
            Err(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Error serializing websocket message: {}", _err);
            }
        }

        ready(())
    }
}

fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    executor: Executor<TCtx, TokioRuntime>,
    ctx_fn: TCtxFn,
    req: httpz::Request,
) -> impl HttpResponse
where
    TCtx: Clone + Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    if !req.server().supports_websockets() {
        #[cfg(feature = "tracing")]
        tracing::debug!("Websocket are not supported on your webserver!");

        // TODO: Make this error be picked up on the frontend and expose it with a logical name
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![])?);
    } else {
        #[cfg(feature = "tracing")]
        tracing::debug!("Accepting websocket connection");
    }

    // TODO: Remove need for `_internal_dangerously_clone`
    let ctx = match ctx_fn.exec(req._internal_dangerously_clone(), None) {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(vec![])?);
        }
    };

    let cookies = req.cookies(); // TODO: Reorder args of next func so cookies goes first
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |_, socket| async move {
        let (tx, mut rx) = socket.split(); // TODO: Can httpz do this in a more efficient manner -> It's doing locking internally cause it's agnostic to `Stream`
        let subscription_manager = WebsocketSubscriptionManager(
            Arc::new(Mutex::new(SubscriptionMap::<TokioRuntime>::new())),
            Arc::new(tokio::sync::Mutex::new(tx)),
        );

        while let Some(msg) = rx.next().await {
            match msg {
                Ok(msg) => {
                    let res = match msg {
                        Message::Text(text) => serde_json::from_str::<Value>(&text),
                        Message::Binary(binary) => serde_json::from_slice(&binary),
                        Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                            continue;
                        }
                        Message::Frame(_) => unreachable!(),
                    };

                    match res.and_then(|v| match v.is_array() {
                        true => serde_json::from_value::<Vec<exec::Request>>(v),
                        false => serde_json::from_value::<exec::Request>(v).map(|v| vec![v]),
                    }) {
                        Ok(reqs) => {
                            let responses = executor
                                .execute_batch(
                                    ctx.clone(),
                                    reqs,
                                    Some(subscription_manager.clone()),
                                )
                                .await;

                            let msg = match serde_json::to_string(&responses) {
                                Ok(v) => v,
                                Err(_err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!(
                                        "Error serializing websocket message: {}",
                                        _err
                                    );

                                    continue;
                                }
                            };

                            subscription_manager
                                .1
                                .lock()
                                .await
                                .send(Message::Text(msg))
                                .await
                                .map_err(|_err| {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!(
                                        "Error serializing websocket message: {}",
                                        _err
                                    );
                                })
                                .ok();
                        }
                        Err(_err) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error parsing websocket message: {}", _err);

                            // TODO: Send report of error to frontend

                            continue;
                        }
                    };
                }
                Err(_err) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Error in websocket: {}", _err);

                    // TODO: Send report of error to frontend

                    continue;
                }
            }
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Shutting down websocket connection");

        // TODO: Terminate all subscriptions
    })
    .into_response()
}
