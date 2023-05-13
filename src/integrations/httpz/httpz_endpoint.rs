use futures::{SinkExt, StreamExt};
use futures_channel::mpsc;
use httpz::{
    http::{Method, Response, StatusCode},
    ws::{Message, WebsocketUpgrade},
    Endpoint, GenericEndpoint, HttpEndpoint, HttpResponse,
};
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    internal::{
        jsonrpc::{self, handle_json_rpc, RequestId, SubscriptionSender},
        middleware::ProcedureKind,
    },
    CompiledRouter,
};

use super::*;

impl<TCtx> CompiledRouter<TCtx>
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
                        (&Method::POST, "_batch") => handle_http_batch(ctx_fn, req, &router)
                            .await
                            .into_response(),
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

#[allow(clippy::unwrap_used)] // TODO: Remove this
async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    kind: ProcedureKind,
    req: httpz::Request,
    router: &Arc<CompiledRouter<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let procedure_name = match req.server() {
        #[cfg(feature = "vercel")]
        httpz::Server::Vercel => req
            .query_pairs()
            .and_then(|mut pairs| pairs.find(|e| e.0 == "rspc"))
            .map(|(_, v)| v.to_string()),
        _ => Some(req.uri().path()[1..].to_string()), // Has to be allocated because `TCtxFn` takes ownership of `req`
    }
    .unwrap();

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

    let cookie_jar = Arc::new(Mutex::new(cookies));
    let old_cookies = req.cookies().clone();
    let ctx = ctx_fn.exec(req, Some(CookieJar::new(cookie_jar.clone())));

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
                // TODO: Props just return `None` here so that we don't allocate or need a clone.
                old_cookies, // If cookies were set in the context function they will be lost but it errored so thats probs fine.
            ));
        }
    };

    let mut response = None as Option<jsonrpc::Response>;
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
                        // TODO: Props just return `None` here so that we don't allocate or need a clone.
                        old_cookies, // If cookies were set in the context function they will be lost but it errored so thats probs fine.
                    ));
                }
            },
        },
        Cow::Borrowed(router),
        &mut response,
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

    debug_assert!(response.is_some()); // This would indicate a bug in rspc's jsonrpc_exec code
    let resp = match response {
        Some(resp) => match serde_json::to_vec(&resp) {
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
        // This case is unreachable but an error is here just incase.
        None => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(b"[]".to_vec())?,
    };

    Ok((resp, cookies))
}

#[allow(clippy::unwrap_used)] // TODO: Remove this
async fn handle_http_batch<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: httpz::Request,
    router: &Arc<CompiledRouter<TCtx>>,
) -> impl HttpResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let cookies = req.cookies();
    match serde_json::from_slice::<Vec<jsonrpc::Request>>(req.body()) {
        Ok(reqs) => {
            let cookie_jar = Arc::new(Mutex::new(cookies));
            let old_cookies = req.cookies().clone();

            let mut responses = Vec::with_capacity(reqs.len());
            for op in reqs {
                // TODO: Make `TCtx` require clone and only run the ctx function once for the whole batch.
                let ctx = ctx_fn.exec(
                    req._internal_dangerously_clone(),
                    Some(CookieJar::new(cookie_jar.clone())),
                );

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
                            // TODO: Props just return `None` here so that we don't allocate or need a clone.
                            old_cookies, // If cookies were set in the context function they will be lost but it errored so thats probs fine.
                        ));
                    }
                };

                // #[cfg(feature = "tracing")]
                // tracing::debug!(
                //     "Executing operation '{}' with key '{}' with params {:?}",
                //     kind.to_str(),
                //     procedure_name,
                //     input
                // );

                // TODO: Probs catch panics so they don't take out the whole batch
                let mut response = None as Option<jsonrpc::Response>;
                handle_json_rpc(ctx, op, Cow::Borrowed(router), &mut response).await;
                debug_assert!(response.is_some()); // This would indicate a bug in rspc's jsonrpc_exec code
                if let Some(response) = response {
                    responses.push(response);
                }
            }

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

fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    req: httpz::Request,
    router: Arc<CompiledRouter<TCtx>>,
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
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |req, mut socket| async move {
        let mut subscriptions = HashMap::new();
        let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

        loop {
            tokio::select! {
                biased; // Note: Order is important here
                msg = rx.next() => {
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
                                        let ctx = ctx_fn.exec(req._internal_dangerously_clone(), None);
                                        handle_json_rpc(match ctx {
                                            Ok(v) => v,
                                            Err(_err) => {
                                                #[cfg(feature = "tracing")]
                                                tracing::error!("Error executing context function: {}", _err);

                                                continue;
                                            }
                                        }, request, Cow::Borrowed(&router), SubscriptionSender(&mut tx, &mut subscriptions)
                                        ).await;
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
    })
    .into_response()
}
