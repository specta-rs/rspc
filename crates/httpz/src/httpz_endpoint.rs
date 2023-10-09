use futures::{stream::FuturesUnordered, StreamExt};
use httpz::{
    http::{Method, Response, StatusCode},
    Endpoint, GenericEndpoint, HttpEndpoint, HttpResponse,
};

use serde_json::Value;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use rspc_core::{
    exec::{self, Connection, ExecutorResult},
    Router,
};

use super::{handle_websocket, CookieJar, TCtxFunc};

// TODO: Make this whole file runtime agnostic once httpz is
// TODO: Remove all panics lol
// TODO: Cleanup the code and use more chaining

pub fn endpoint<TCtx, TCtxFnMarker: Send + Sync + 'static, TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>>(
    router: Arc<Router<TCtx>>,
    ctx_fn: TCtxFn,
) -> Endpoint<impl HttpEndpoint>
where
    TCtx: Clone + Send + Sync + 'static,
{
    // TODO: This should be able to call `ctn_fn` prior to the async boundary to avoid cloning it!
    // TODO: Basically httpz would need to be able to return `Response | Future<Response>` basically how rspc executor works.

    GenericEndpoint::new(
        "/:id", // TODO: I think this is Axum specific. Fix in `httpz`!
        [Method::GET, Method::POST],
        move |req: httpz::Request| {
            // TODO: It would be nice if these clones weren't per request.
            // TODO: Maybe httpz can `Box::leak` a ref to a context type and allow it to be shared.
            let router = router.clone();
            let ctx_fn = ctx_fn.clone();

            async move {
                match (req.method(), &req.uri().path()[1..]) {
                    (&Method::GET, "ws") => handle_websocket(router, ctx_fn, req).into_response(),
                    (&Method::GET, _) => handle_http(router, ctx_fn, req).await.into_response(),
                    (&Method::POST, "_batch") => {
                        handle_http_batch(router, ctx_fn, req).await.into_response()
                    }
                    (&Method::POST, _) => handle_http(router, ctx_fn, req).await.into_response(),
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(vec![])?),
                }
            }
        },
    )
}

#[allow(clippy::unwrap_used)] // TODO: Remove all panics lol
async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    router: Arc<Router<TCtx>>,
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

            exec::Request::Query(exec::RequestData { id: 0, path, input })
        }
        Method::POST => {
            let input = (!req.body().is_empty())
                .then(|| serde_json::from_slice(req.body()))
                .unwrap_or(Ok(None))
                .unwrap();

            exec::Request::Mutation(exec::RequestData { id: 0, path, input })
        }
        _ => unreachable!(),
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

    let response =
        match router.execute(ctx, request, None) {
        	Some(res) => match res {
	            ExecutorResult::Future(fut) => fut.await,
	            ExecutorResult::Response(response) => response,
	            ExecutorResult::Task(task) => todo!(),
	        },
            None => unreachable!(
                "Executor will only return none for a 'stopSubscription' event which is impossible here"
            ),
        }.inner;

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
    router: Arc<Router<TCtx>>,
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

            let fut_responses = FuturesUnordered::new();

            let mut responses = Vec::with_capacity(requests.len());
            for req in requests {
                let Some(res) = router.clone().execute(ctx.clone(), req, None) else {
                    continue;
                };

                match res {
                    ExecutorResult::Future(fut) => {
                        fut_responses.push(fut);
                    }
                    ExecutorResult::Response(resp) => {
                        responses.push(resp);
                    }
                    ExecutorResult::Task(task) => todo!(),
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

            responses.append(&mut fut_responses.collect().await);
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
