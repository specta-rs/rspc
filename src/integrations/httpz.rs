use futures::{SinkExt, StreamExt};
use httpz::{
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    Endpoint, GenericEndpoint, HttpEndpoint, HttpResponse,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    mem,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

use crate::{
    internal::{
        jsonrpc::{self, handle_json_rpc, RequestId, Sender, SubscriptionMap},
        ProcedureKind,
    },
    Router,
};

pub use super::httpz_extractors::*;
pub use httpz::cookie::Cookie;

/// TODO
///
// TODO: Can `Rc<RefCell<T>>` be used so I don't need to await a borrow and use Tokio specific API's???
// TODO: The `Mutex` will block. This isn't great, work to remove it. The Tokio `Mutex` makes everything annoyingly async so I don't use it.
#[derive(Debug)]
pub struct CookieJar(Arc<Mutex<httpz::cookie::CookieJar>>);

impl CookieJar {
    pub(super) fn new(cookies: Arc<Mutex<httpz::cookie::CookieJar>>) -> Self {
        Self(cookies)
    }

    /// Returns a reference to the `Cookie` inside this jar with the name
    /// `name`. If no such cookie exists, returns `None`.
    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        self.0.lock().unwrap().get(name).cloned() // TODO: `cloned` is cringe avoid it by removing `Mutex`?
    }

    /// Adds an "original" `cookie` to this jar. If an original cookie with the
    /// same name already exists, it is replaced with `cookie`. Cookies added
    /// with `add` take precedence and are not replaced by this method.
    ///
    /// Adding an original cookie does not affect the [delta](#method.delta)
    /// computation. This method is intended to be used to seed the cookie jar
    /// with cookies received from a client's HTTP message.
    ///
    /// For accurate `delta` computations, this method should not be called
    /// after calling `remove`.
    pub fn add_original(&self, cookie: Cookie<'static>) {
        self.0.lock().unwrap().add_original(cookie)
    }

    /// Adds `cookie` to this jar. If a cookie with the same name already
    /// exists, it is replaced with `cookie`.
    pub fn add(&self, cookie: Cookie<'static>) {
        self.0.lock().unwrap().add(cookie);
    }

    /// Removes `cookie` from this jar. If an _original_ cookie with the same
    /// name as `cookie` is present in the jar, a _removal_ cookie will be
    /// present in the `delta` computation. To properly generate the removal
    /// cookie, `cookie` must contain the same `path` and `domain` as the cookie
    /// that was initially set.
    ///
    /// A "removal" cookie is a cookie that has the same name as the original
    /// cookie but has an empty value, a max-age of 0, and an expiration date
    /// far in the past. See also [`Cookie::make_removal()`].
    ///
    /// Removing a new cookie does not result in a _removal_ cookie unless
    /// there's an original cookie with the same name:
    pub fn remove(&self, cookie: Cookie<'static>) {
        self.0.lock().unwrap().remove(cookie)
    }

    /// Removes `cookie` from this jar completely. This method differs from
    /// `remove` in that no delta cookie is created under any condition. Neither
    /// the `delta` nor `iter` methods will return a cookie that is removed
    /// using this method.
    pub fn force_remove<'a>(&self, cookie: &Cookie<'a>) {
        self.0.lock().unwrap().force_remove(cookie)
    }

    /// Removes all delta cookies, i.e. all cookies not added via
    /// [`CookieJar::add_original()`], from this `CookieJar`. This undoes any
    /// changes from [`CookieJar::add()`] and [`CookieJar::remove()`]
    /// operations.
    pub fn reset_delta(&self) {
        self.0.lock().unwrap().reset_delta()
    }

    // /// Returns an iterator over cookies that represent the changes to this jar
    // /// over time. These cookies can be rendered directly as `Set-Cookie` header
    // /// values to affect the changes made to this jar on the client.
    // pub fn delta(&self) -> Delta {
    //     self.0.lock().unwrap().delta()
    // }

    // /// Returns an iterator over all of the cookies present in this jar.
    // pub fn iter(&self) -> Iter {
    //     self.0.lock().unwrap().iter()
    // }

    // /// Returns a read-only `PrivateJar` with `self` as its parent jar using the
    // /// key `key` to verify/decrypt cookies retrieved from the child jar. Any
    // /// retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "private")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "private")))]
    // pub fn private<'a>(&'a self, key: &Key) -> PrivateJar<&'a Self> {
    //     PrivateJar::new(self, key)
    // }

    // /// Returns a read/write `PrivateJar` with `self` as its parent jar using
    // /// the key `key` to sign/encrypt and verify/decrypt cookies added/retrieved
    // /// from the child jar.
    // ///
    // /// Any modifications to the child jar will be reflected on the parent jar,
    // /// and any retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "private")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "private")))]
    // pub fn private_mut<'a>(&'a mut self, key: &Key) -> PrivateJar<&'a mut Self> {
    //     PrivateJar::new(self, key)
    // }

    // /// Returns a read-only `SignedJar` with `self` as its parent jar using the
    // /// key `key` to verify cookies retrieved from the child jar. Any retrievals
    // /// from the child jar will be made from the parent jar.
    // #[cfg(feature = "signed")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "signed")))]
    // pub fn signed<'a>(&'a self, key: &Key) -> SignedJar<&'a Self> {
    //     SignedJar::new(self, key)
    // }

    // /// Returns a read/write `SignedJar` with `self` as its parent jar using the
    // /// key `key` to sign/verify cookies added/retrieved from the child jar.
    // ///
    // /// Any modifications to the child jar will be reflected on the parent jar,
    // /// and any retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "signed")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "signed")))]
    // pub fn signed_mut<'a>(&'a mut self, key: &Key) -> SignedJar<&'a mut Self> {
    //     SignedJar::new(self, key)
    // }
}

/// TODO
///
/// This wraps [httpz::Request] removing any methods that are not safe with rspc such as `body`, `into_parts` and replacing the cookie handling API.
///
#[derive(Debug)]
pub struct Request(httpz::Request, Option<CookieJar>);

impl Request {
    pub(crate) fn new(req: httpz::Request, cookies: Option<CookieJar>) -> Self {
        Self(req, cookies)
    }

    /// Get the uri of the request.
    pub fn uri(&self) -> &httpz::http::Uri {
        self.0.uri()
    }

    /// Get the version of the request.
    pub fn version(&self) -> httpz::http::Version {
        self.0.version()
    }

    /// Get the method of the request.
    pub fn method(&self) -> &httpz::http::Method {
        self.0.method()
    }

    /// Get the headers of the request.
    pub fn headers(&self) -> &httpz::http::HeaderMap {
        self.0.headers()
    }

    /// TODO
    pub fn cookies(&mut self) -> Option<CookieJar> {
        // TODO: This take means a `None` response could be because it was already used or because it's a websocket. This is a confusing DX and needs fixing.

        mem::replace(&mut self.1, None)
    }

    /// query_pairs returns an iterator of the query parameters.
    pub fn query_pairs(&self) -> Option<httpz::form_urlencoded::Parse<'_>> {
        self.0.query_pairs()
    }

    /// TODO
    pub fn server(&self) -> httpz::Server {
        self.0.server()
    }

    // TODO: Downcasting extensions both `mut` and `ref`
    // TODO: Inserting extensions
}

impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub fn endpoint<TCtxFnMarker: Send + Sync + 'static, TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>>(
        self: Arc<Self>,
        ctx_fn: TCtxFn,
    ) -> Endpoint<impl HttpEndpoint> {
        GenericEndpoint::new(
            "/:id", // TODO: I think this is Axum specific. Fix in `httpz`!
            [Method::GET, Method::POST],
            move |req: httpz::Request| {
                // TODO: It would be nice if these clones weren't per request.
                // TODO: Maybe httpz can `Box::leak` a ref to a context type and allow it to be shared.
                let router = self.clone();
                let ctx_fn = ctx_fn.clone();

                async move {
                    match (req.method(), &req.uri().path()[1..]) {
                        (&Method::GET, "ws") => {
                            handle_websocket(ctx_fn, req, router).into_response()
                        }
                        (&Method::GET, _) => {
                            handle_http(ctx_fn, ProcedureKind::Query, req, &router)
                                .await
                                .into_response()
                        }
                        (&Method::POST, _) => {
                            handle_http(ctx_fn, ProcedureKind::Mutation, req, &router)
                                .await
                                .into_response()
                        }
                        _ => unreachable!(),
                    }
                }
            },
        )
    }
}

pub async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    kind: ProcedureKind,
    mut req: httpz::Request,
    router: &Arc<Router<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let procedure_name = req.uri().path()[1..].to_string(); // Has to be allocated because `TCtxFn` takes ownership of `req`
    let input = match *req.method() {
        Method::GET => req
            .query_pairs()
            .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => (!req.body().is_empty())
            .then(|| serde_json::from_slice(req.body()))
            .unwrap_or(Ok(None)),
        _ => unreachable!(),
    };
    let cookies = req.cookies();

    let input = match input {
        Ok(input) => input,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!(
                "Error passing parameters to operation '{}' with key '{:?}': {}",
                kind.to_str(),
                procedure_name,
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
        procedure_name,
        input
    );

    let mut resp = Sender::Response(None);

    let cookie_jar = Arc::new(Mutex::new(cookies));
    let ctx = ctx_fn.exec(&mut req, Some(CookieJar::new(cookie_jar.clone())));

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
                req.cookies(), // If cookies were set in the context function they will be lost but it errored so thats probs fine.
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
                    path: procedure_name.to_string(), // TODO: Lifetime instead of allocate?
                    input,
                },
                ProcedureKind::Mutation => jsonrpc::RequestInner::Mutation {
                    path: procedure_name.to_string(), // TODO: Lifetime instead of allocate?
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
                        req.cookies(), // If cookies were set in the context function they will be lost but it errored so thats probs fine.
                    ));
                }
            },
        },
        router,
        &mut resp,
        &mut SubscriptionMap::None,
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

pub fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: httpz::Request,
    router: Arc<Router<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    #[cfg(feature = "tracing")]
    tracing::debug!("Accepting websocket connection");

    if !req.server().supports_websockets() {
        #[cfg(feature = "tracing")]
        tracing::debug!("Websocket are not supported on your webserver!");

        // TODO: Make this error be picked up on the frontend and expose it with a logical name
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![])?);
    }

    let cookies = req.cookies();
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |mut req, mut socket| async move {
        let mut subscriptions = HashMap::new();
        let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

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
                    match msg {
                        Some(Ok(msg) )=> {
                           let res = match msg {
                                Message::Text(text) => serde_json::from_str::<Value>(&text),
                                Message::Binary(binary) => serde_json::from_slice(&binary),
                                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                                    continue;
                                }
                                Message::Frame(_) => unreachable!(),
                            };

                            match res.and_then(|v| match v.is_array() {
                                    true => serde_json::from_value::<Vec<jsonrpc::Request>>(v),
                                    false => serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v]),
                                }) {
                                Ok(reqs) => {
                                    for request in reqs {
                                        let ctx = ctx_fn.exec(&mut req, None);

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
                                },
                                Err(_err) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("Error parsing websocket message: {}", _err);

                                    // TODO: Send report of error to frontend

                                    continue;
                                }
                            };
                        }
                        Some(Err(_err)) => {
                            #[cfg(feature = "tracing")]
                            tracing::error!("Error in websocket: {}", _err);

                            // TODO: Send report of error to frontend

                            continue;
                        },
                        None => {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Shutting down websocket connection");

                            // TODO: Send report of error to frontend

                            return;
                        },
                    }
                }
            }
        }
    }).into_response()
}
