use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    body::{Bytes, HttpBody},
    extract::{FromRequest, Request},
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::{on, MethodFilter},
};
use futures::Stream;
use rspc_core::{ProcedureStream, Procedures};

/// Construct a new [`axum::Router`](axum::Router) to expose a given [`rspc::Router`](rspc::Router).
pub struct Endpoint<TCtx> {
    procedures: Procedures<TCtx>,
    // endpoints: bool,
    // websocket: Option<fn(&TCtx) -> TCtx>,
    // batching: bool,
}

impl<TCtx: Send + 'static> Endpoint<TCtx> {
    // /// Construct a new [`axum::Router`](axum::Router) with all features enabled.
    // ///
    // /// This will enable all features, if you want to configure which features are enabled you can use [`Endpoint::builder`] instead.
    // ///
    // /// # Usage
    // ///
    // /// ```rust
    // /// axum::Router::new().nest(
    // ///     "/rspc",
    // ///     rspc_axum::Endpoint::new(rspc::Router::new().build().unwrap(), || ()),
    // /// );
    // /// ```
    // pub fn new<S>(
    //     router: BuiltRouter<TCtx>,
    //     // TODO: Parse this to `Self::build` -> It will make rustfmt result way nicer
    //     // TODO: Make Axum extractors work
    //     ctx_fn: impl Fn(&Parts) -> TCtx + Send + Sync + 'static,
    // ) -> axum::Router<S>
    // where
    //     S: Clone + Send + Sync + 'static,
    //     // TODO: Error type???
    //     // F: Future<Output = Result<TCtx, ()>> + Send + Sync + 'static,
    //     TCtx: Clone,
    // {
    //     let mut t = Self::builder(router).with_endpoints();
    //     #[cfg(feature = "ws")]
    //     {
    //         t = t.with_websocket();
    //     }
    //     t.with_batching().build(ctx_fn)
    // }

    // /// Construct a new [`Endpoint`](Endpoint) with no features enabled.
    // ///
    // /// # Usage
    // ///
    // /// ```rust
    // /// axum::Router::new().nest(
    // ///     "/rspc",
    // ///     rspc_axum::Endpoint::builder(rspc::Router::new().build().unwrap())
    // ///         // Exposes HTTP endpoints for queries and mutations.
    // ///         .with_endpoints()
    // ///         // Exposes a Websocket connection for queries, mutations and subscriptions.
    // ///         .with_websocket()
    // ///         // Enables support for the frontend sending batched queries.
    // ///         .with_batching()
    // ///         .build(|| ()),
    // /// );
    // /// ```
    pub fn builder(router: Procedures<TCtx>) -> Self {
        Self {
            procedures: router,
            // endpoints: false,
            // websocket: None,
            // batching: false,
        }
    }

    // /// Enables HTTP endpoints for queries and mutations.
    // ///
    // /// This is exposed as `/routerName.procedureName`
    // pub fn with_endpoints(mut self) -> Self {
    //     Self {
    //         endpoints: true,
    //         ..self
    //     }
    // }

    // /// Exposes a Websocket connection for queries, mutations and subscriptions.
    // ///
    // /// This is exposed as a `/ws` endpoint.
    // #[cfg(feature = "ws")]
    // #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    // pub fn with_websocket(self) -> Self
    // where
    //     TCtx: Clone,
    // {
    //     Self {
    //         websocket: Some(|ctx| ctx.clone()),
    //         ..self
    //     }
    // }

    // /// Enables support for the frontend sending batched queries.
    // ///
    // /// This is exposed as a `/_batch` endpoint.
    // pub fn with_batching(self) -> Self
    // where
    //     TCtx: Clone,
    // {
    //     Self {
    //         batching: true,
    //         ..self
    //     }
    // }

    // TODO: Axum extractors

    /// Build an [`axum::Router`](axum::Router) with the configured features.
    pub fn build<S>(self, ctx_fn: impl Fn() -> TCtx + Send + Sync + 'static) -> axum::Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        let mut r = axum::Router::new();
        let ctx_fn = Arc::new(ctx_fn);

        for (key, procedure) in self.procedures {
            let ctx_fn = ctx_fn.clone();
            r = r.route(
                &format!("/{key}"),
                on(
                    MethodFilter::GET.or(MethodFilter::POST),
                    move |req: Request| {
                        let ctx = ctx_fn();

                        async move {
                            let hint = req.body().size_hint();
                            let has_body = hint.lower() != 0 || hint.upper() != Some(0);
                            let stream = if !has_body {
                                let mut params = form_urlencoded::parse(
                                    req.uri().query().unwrap_or_default().as_ref(),
                                );

                                match params
                                    .find_map(|(input, value)| (input == "input").then(|| value))
                                {
                                    Some(input) => procedure.exec_with_deserializer(
                                        ctx,
                                        &mut serde_json::Deserializer::from_str(&*input),
                                    ),
                                    None => procedure
                                        .exec_with_deserializer(ctx, serde_json::Value::Null),
                                }
                            } else {
                                // TODO
                                // if !json_content_type(req.headers()) {
                                //     Err(MissingJsonContentType.into())
                                // }

                                let bytes = Bytes::from_request(req, &()).await.unwrap(); // TODO: Error handling
                                procedure.exec_with_deserializer(
                                    ctx,
                                    &mut serde_json::Deserializer::from_slice(&bytes),
                                )
                            };

                            // TODO: Status code
                            // TODO: Json headers
                            // TODO: Maybe only SSE for subscriptions???

                            Sse::new(ProcedureStreamSSE(stream)).keep_alive(KeepAlive::default())
                        }
                    },
                ),
            );
        }

        // TODO: Websocket endpoint

        r
    }
}

struct ProcedureStreamSSE(ProcedureStream);

impl Stream for ProcedureStreamSSE {
    type Item = Result<Event, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0
            .poll_next(cx)
            // TODO: `v` should be broken out
            .map(|v| {
                v.map(|v| {
                    Event::default()
                        // .event() // TODO: `Ok` vs `Err` - Also serve `StatusCode`
                        .json_data(&v)
                })
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// fn json_content_type(headers: &HeaderMap) -> bool {
//     let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
//         content_type
//     } else {
//         return false;
//     };

//     let content_type = if let Ok(content_type) = content_type.to_str() {
//         content_type
//     } else {
//         return false;
//     };

//     let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
//         mime
//     } else {
//         return false;
//     };

//     let is_json_content_type = mime.type_() == "application"
//         && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

//     is_json_content_type
// }
