//! rspc-actix-web: [Actix Web](https://actix.rs) integration for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use actix_web::{
    web::{self, ServiceConfig},
    HttpResponse, Resource,
};
use rspc_core::Procedures;

pub struct Endpoint<TCtx> {
    procedures: Procedures<TCtx>,
    // endpoints: bool,
    // websocket: Option<fn(&TCtx) -> TCtx>,
    // batching: bool,
}

impl<TCtx: Send + 'static> Endpoint<TCtx> {
    pub fn builder(router: Procedures<TCtx>) -> Self {
        Self {
            procedures: router,
            // endpoints: false,
            // websocket: None,
            // batching: false,
        }
    }

    pub fn build(
        self,
        ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static,
    ) -> impl FnOnce(&mut ServiceConfig) {
        |service| {
            service.route(
                "/ws",
                web::to(|| {
                    // let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

                    //    let mut stream = stream
                    //        .aggregate_continuations()
                    //        // aggregate continuation frames up to 1MiB
                    //        .max_continuation_size(2_usize.pow(20));

                    //    // start task but don't wait for it
                    //    rt::spawn(async move {
                    //        // receive messages from websocket
                    //        while let Some(msg) = stream.next().await {
                    //            match msg {
                    //                Ok(AggregatedMessage::Text(text)) => {
                    //                    // echo text message
                    //                    session.text(text).await.unwrap();
                    //                }

                    //                Ok(AggregatedMessage::Binary(bin)) => {
                    //                    // echo binary message
                    //                    session.binary(bin).await.unwrap();
                    //                }

                    //                Ok(AggregatedMessage::Ping(msg)) => {
                    //                    // respond to PING frame with PONG frame
                    //                    session.pong(&msg).await.unwrap();
                    //                }

                    //                _ => {}
                    //            }
                    //        }

                    HttpResponse::NotFound()
                }),
            );
            service.route("/{route:.*}", web::to(|| HttpResponse::Ok()));
        }
    }
}
