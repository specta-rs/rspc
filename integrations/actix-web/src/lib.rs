//! rspc-actix-web: [Actix Web](https://actix.rs) integration for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

use std::{convert::Infallible, sync::Arc};

use actix_web::{
    body::BodyStream,
    http::{header, StatusCode},
    web::{self, Bytes, Payload, ServiceConfig},
    HttpRequest, HttpResponse,
};
use actix_ws::Message;

use futures_util::StreamExt;
use rspc_core::Procedures;
use rspc_http::ExecuteInput;

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
        let ctx_fn = Arc::new(ctx_fn);
        move |service| {
            service.route(
                "/ws",
                // TODO: Hook this up properly
                web::to(|req: HttpRequest, body: Payload| async move {
                    let (response, mut session, mut stream) = actix_ws::handle(&req, body)?;

                    actix_web::rt::spawn(async move {
                        session.text("Hello World From rspc").await.unwrap();

                        while let Some(Ok(msg)) = stream.next().await {
                            match msg {
                                Message::Ping(bytes) => {
                                    if session.pong(&bytes).await.is_err() {
                                        return;
                                    }
                                }

                                Message::Text(msg) => println!("Got text: {msg}"),
                                _ => break,
                            }
                        }

                        let _ = session.close(None).await;
                    });

                    Ok::<_, actix_web::Error>(response)
                }),
            );

            // TODO: Making extractors work

            for (key, procedure) in self.procedures {
                let ctx_fn = ctx_fn.clone();
                let handler = move |req: HttpRequest, body: Bytes| {
                    let procedure = procedure.clone();
                    let ctx_fn = ctx_fn.clone();
                    async move {
                        let input = if body.is_empty() {
                            ExecuteInput::Query(req.query_string())
                        } else {
                            // TODO: Error if not JSON content-type

                            ExecuteInput::Body(&body)
                        };

                        let (status, stream) =
                            rspc_http::execute(&procedure, input, || ctx_fn()).await;
                        HttpResponse::build(
                            StatusCode::from_u16(status)
                                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        )
                        .insert_header(header::ContentType::json())
                        .body(BodyStream::new(
                            stream.map(|v| Ok::<_, Infallible>(v.into())),
                        ))
                    }
                };

                service.route(&key, web::get().to(handler.clone()));
                service.route(&key, web::post().to(handler));
            }
        }
    }
}

// pub struct TODO(HttpRequest);

// impl rspc_http::Request for TODO {
//     fn method(&self) -> &str {
//         self.0.method().as_str()
//     }

//     // fn path(&self) -> &str {
//     //     self.0.path()
//     // }

//     // fn query(&self) -> &str {
//     //     self.0.query_string()
//     // }

//     // fn body(&self) {
//     //     self.0.
//     // }
// }
