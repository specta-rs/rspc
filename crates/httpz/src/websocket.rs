use std::{pin::pin, sync::Arc};

use futures::{SinkExt, StreamExt};
use httpz::{
    http,
    ws::{Message, WebsocketUpgrade},
    HttpResponse,
};
use rspc_core::{
    exec::{run_connection, IncomingMessage, Response},
    Router,
};

use super::TCtxFunc;

pub(crate) fn handle_websocket<TCtx, TCtxFn, TCtxFnMarker>(
    router: Arc<Router<TCtx>>,
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
        return Ok(http::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
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

            return Ok(http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(vec![])?);
        }
    };

    let cookies = req.cookies(); // TODO: Reorder args of next func so cookies goes first
    WebsocketUpgrade::from_req_with_cookies(req, cookies, move |_, socket| async move {
        let socket = socket
            .with(|v: Vec<Response>| async move {
                Ok(Message::Text(serde_json::to_string(&v).unwrap())) as Result<_, httpz::Error>
            })
            .map(|v| {
                v.map(|v| match v {
                    Message::Text(v) => IncomingMessage::Msg(serde_json::from_str(&v)),
                    Message::Binary(v) => IncomingMessage::Msg(serde_json::from_slice(&v)),
                    Message::Ping(_) | Message::Pong(_) => IncomingMessage::Skip,
                    // TODO: This is a suboptimal feature flag cause it's *akshually* based on Tokio or not.
                    #[cfg(feature = "axum")]
                    Message::Close(_) => IncomingMessage::Close,
                    // TODO: This is a suboptimal feature flag cause it's *akshually* based on Tokio or not.
                    #[cfg(feature = "axum")]
                    Message::Frame(_) => {
                        #[cfg(debug_assertions)]
                        unreachable!("Reading a 'httpz::ws::Message::Frame' is impossible");

                        #[cfg(not(debug_assertions))]
                        return IncomingMessage::Skip;
                    }
                })
            });

        run_connection(ctx, router, pin!(socket), None).await;
    })
    .into_response()
}
