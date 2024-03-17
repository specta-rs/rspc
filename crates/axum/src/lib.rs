//! rspc-axum: Axum integration for [rspc](https://rspc.dev).

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{Method, Response, StatusCode},
    response::IntoResponse,
    routing::{on, MethodFilter},
    Router,
};
use extractors::TCtxFunc;
use rspc::internal::{
    jsonrpc::{self, handle_json_rpc, RequestId, Sender, SubscriptionMap},
    ProcedureKind,
};
use serde_json::Value;

mod extractors;

pub fn endpoint<TCtx, TCtxFnMarker, TCtxFn, S>(
    router: Arc<rspc::Router<TCtx>>,
    ctx_fn: TCtxFn,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TCtxFnMarker: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    Router::<S>::new().route(
        "/:id",
        on(
            MethodFilter::GET.or(MethodFilter::POST),
            move |req: axum::extract::Request<Body>| {
                let router = router.clone();

                async move {
                    let ctx_fn = ctx_fn.clone();

                    match (req.method(), &req.uri().path()[1..]) {
                        // (&Method::GET, "ws") => handle_websocket(ctx_fn, req, router).into_response(),
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
        ),
    )
}

pub async fn handle_http<TCtx, TCtxFn, TCtxFnMarker>(
    ctx_fn: TCtxFn,
    kind: ProcedureKind,
    req: Request,
    router: &Arc<rspc::Router<TCtx>>,
) -> impl IntoResponse
where
    TCtx: Send + Sync + 'static,
    TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
{
    let procedure_name = req.uri().path()[1..].to_string(); // Has to be allocated because `TCtxFn` takes ownership of `req`
    let (parts, body) = req.into_parts();
    let input = match parts.method {
        Method::GET => parts
            .uri
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .and_then(|mut params| params.find(|e| e.0 == "input").map(|e| e.1))
            .map(|v| serde_json::from_str(&v))
            .unwrap_or(Ok(None as Option<Value>)),
        Method::POST => {
            // TODO: Limit body size?
            let body = to_bytes(body, usize::MAX).await.unwrap(); // TODO: error handling
            (!body.is_empty())
                .then(|| serde_json::from_slice(body.to_vec().as_slice()))
                .unwrap_or(Ok(None))
        }
        _ => unreachable!(),
    };

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

            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Body::from(b"[]".as_slice()))
                .unwrap();
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

    let ctx = ctx_fn.exec(parts);

    let ctx = match ctx {
        Ok(v) => v,
        Err(_err) => {
            #[cfg(feature = "tracing")]
            tracing::error!("Error executing context function: {}", _err);

            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(Body::from(b"[]".as_slice()))
                .unwrap();
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

                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "application/json")
                        .body(Body::from(b"[]".as_slice()))
                        .unwrap();
                }
            },
        },
        router,
        &mut resp,
        &mut SubscriptionMap::None,
    )
    .await;

    match resp {
        Sender::Response(Some(resp)) => match serde_json::to_vec(&resp) {
            Ok(v) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(v))
                .unwrap(),
            Err(_err) => {
                #[cfg(feature = "tracing")]
                tracing::error!("Error serializing response: {}", _err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Body::from(b"[]".as_slice()))
                    .unwrap()
            }
        },
        _ => unreachable!(),
    }
}

// pub fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
//     ctx_fn: TCtxFn,
//     req: httpz::Request,
//     router: Arc<Router<TCtx>>,
// ) -> impl HttpResponse
// where
//     TCtx: Send + Sync + 'static,
//     TCtxFn: TCtxFunc<TCtx, TCtxFnMarker>,
// {
//     #[cfg(feature = "tracing")]
//     tracing::debug!("Accepting websocket connection");

//     if !req.server().supports_websockets() {
//         #[cfg(feature = "tracing")]
//         tracing::debug!("Websocket are not supported on your webserver!");

//         // TODO: Make this error be picked up on the frontend and expose it with a logical name
//         return Ok(Response::builder()
//             .status(StatusCode::INTERNAL_SERVER_ERROR)
//             .body(vec![])?);
//     }

//     let cookies = req.cookies();
//     WebsocketUpgrade::from_req_with_cookies(req, cookies, move |req, mut socket| async move {
//         let mut subscriptions = HashMap::new();
//         let (mut tx, mut rx) = mpsc::channel::<jsonrpc::Response>(100);

//         loop {
//             tokio::select! {
//                 biased; // Note: Order is important here
//                 msg = rx.recv() => {
//                     match socket.send(Message::Text(match serde_json::to_string(&msg) {
//                         Ok(v) => v,
//                         Err(_err) => {
//                             #[cfg(feature = "tracing")]
//                             tracing::error!("Error serializing websocket message: {}", _err);

//                             continue;
//                         }
//                     })).await {
//                         Ok(_) => {}
//                         Err(_err) => {
//                             #[cfg(feature = "tracing")]
//                             tracing::error!("Error sending websocket message: {}", _err);

//                             continue;
//                         }
//                     }
//                 }
//                 msg = socket.next() => {
//                     match msg {
//                         Some(Ok(msg) )=> {
//                            let res = match msg {
//                                 Message::Text(text) => serde_json::from_str::<Value>(&text),
//                                 Message::Binary(binary) => serde_json::from_slice(&binary),
//                                 Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
//                                     continue;
//                                 }
//                                 Message::Frame(_) => unreachable!(),
//                             };

//                             match res.and_then(|v| match v.is_array() {
//                                     true => serde_json::from_value::<Vec<jsonrpc::Request>>(v),
//                                     false => serde_json::from_value::<jsonrpc::Request>(v).map(|v| vec![v]),
//                                 }) {
//                                 Ok(reqs) => {
//                                     for request in reqs {
//                                         let ctx = ctx_fn.exec(req._internal_dangerously_clone(), None);

//                                             handle_json_rpc(match ctx {
//                                                 Ok(v) => v,
//                                                 Err(_err) => {
//                                                     #[cfg(feature = "tracing")]
//                                                     tracing::error!("Error executing context function: {}", _err);

//                                                     continue;
//                                                 }
//                                             }, request, &router, &mut Sender::Channel(&mut tx),
//                                             &mut SubscriptionMap::Ref(&mut subscriptions)).await;
//                                     }
//                                 },
//                                 Err(_err) => {
//                                     #[cfg(feature = "tracing")]
//                                     tracing::error!("Error parsing websocket message: {}", _err);

//                                     // TODO: Send report of error to frontend

//                                     continue;
//                                 }
//                             };
//                         }
//                         Some(Err(_err)) => {
//                             #[cfg(feature = "tracing")]
//                             tracing::error!("Error in websocket: {}", _err);

//                             // TODO: Send report of error to frontend

//                             continue;
//                         },
//                         None => {
//                             #[cfg(feature = "tracing")]
//                             tracing::debug!("Shutting down websocket connection");

//                             // TODO: Send report of error to frontend

//                             return;
//                         },
//                     }
//                 }
//             }
//         }
//     }).into_response()
// }
