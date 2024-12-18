use std::{
    convert::Infallible,
    future::poll_fn,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{FromRequest, Request},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{on, MethodFilter},
};
use futures::{stream::once, Stream, StreamExt, TryStreamExt};
use rspc_core::{ProcedureError, ProcedureStream, Procedures};

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
                            let mut stream = if !has_body {
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
                                if !json_content_type(req.headers()) {
                                    let err: ProcedureError = rspc_core::DeserializeError::custom(
                                        "Client did not set correct valid 'Content-Type' header",
                                    )
                                    .into();
                                    let buf = serde_json::to_vec(&err).unwrap(); // TODO

                                    return (
                                        StatusCode::BAD_REQUEST,
                                        [(header::CONTENT_TYPE, "application/json")],
                                        Body::from(buf),
                                    )
                                        .into_response();
                                }

                                let bytes = Bytes::from_request(req, &()).await.unwrap(); // TODO: Error handling
                                procedure.exec_with_deserializer(
                                    ctx,
                                    &mut serde_json::Deserializer::from_slice(&bytes),
                                )
                            };

                            let mut stream = ProcedureStreamResponse {
                                code: None,
                                stream,
                                first: None,
                            };
                            stream.first = Some(stream.next().await);

                            let status = stream
                                .code
                                .and_then(|c| StatusCode::try_from(c).ok())
                                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                            (
                                status,
                                [(header::CONTENT_TYPE, "application/json")],
                                Body::from_stream(stream),
                            )
                                .into_response()
                        }
                    },
                ),
            );
        }

        // TODO: Websocket endpoint

        r
    }
}

struct ProcedureStreamResponse {
    code: Option<u16>,
    first: Option<Option<Result<Vec<u8>, Infallible>>>,
    stream: ProcedureStream,
}

impl Stream for ProcedureStreamResponse {
    type Item = Result<Vec<u8>, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(first) = self.first.take() {
            return Poll::Ready(first);
        }

        let (code, mut buf) = {
            let Poll::Ready(v) = self.stream.poll_next(cx) else {
                return Poll::Pending;
            };

            match v {
                Some(Ok(v)) => (
                    200,
                    Some(serde_json::to_vec(&v).unwrap()), // TODO: Error handling
                ),
                Some(Err(err)) => (
                    err.status(),
                    Some(serde_json::to_vec(&err).unwrap()), // TODO: Error handling
                ),
                None => (200, None),
            }
        };

        if let Some(buf) = &mut buf {
            buf.extend_from_slice(b"\n\n");
        };

        self.code = Some(code);
        Poll::Ready(buf.map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

fn json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

    is_json_content_type
}
