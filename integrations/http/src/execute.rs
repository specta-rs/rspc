//! Stream body types by framework:
//!  Axum - impl Stream<Item = Result<impl Into<bytes::Bytes>, impl Into<Box<dyn Error + Send + Sync>>>
//!  Actix Web - impl Stream<Item = Result<actix_web::web::Bytes, impl Into<Box<dyn Error>> + 'static>
//!  Poem - impl Stream<Item = Result<impl Into<bytes::Bytes>, impl Into<std::io::Error>>>
//!  Warp - impl Stream<Item = Result<impl bytes::buf::Buf, warp::Error>>
//!  Tide - N/A supports impl futures::AsyncBufRead
//!  Hyper (via http_body_util::StreamBody) - impl Stream<Item = Result<Frame<impl bytes::Buf>, E>>,
//!  Rocket - impl Stream<Item = impl AsRef<[u8]>>

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::StreamExt;
use futures_core::Stream;
use rspc_core::ProcedureStream;
use serde::Serializer;

pub enum ExecuteInput<'a> {
    Query(&'a str),
    Body(&'a [u8]),
}

/// TODO: Explain this
// TODO: `Content-Type` header on response???
pub async fn execute<'a, 'b, TCtx>(
    procedure: &'a rspc_core::Procedure<TCtx>,
    input: ExecuteInput<'b>,
    ctx: impl FnOnce() -> TCtx,
) -> (u16, impl Stream<Item = Vec<u8>> + Send + 'static) {
    let stream = match input {
        ExecuteInput::Query(query) => {
            let mut params = form_urlencoded::parse(query.as_bytes());

            match params.find_map(|(input, value)| (input == "input").then(|| value)) {
                Some(input) => procedure.exec_with_deserializer(
                    ctx(),
                    &mut serde_json::Deserializer::from_str(&*input),
                ),
                None => procedure.exec_with_deserializer(ctx(), serde_json::Value::Null),
            }
        }
        ExecuteInput::Body(body) => procedure
            .exec_with_deserializer(ctx(), &mut serde_json::Deserializer::from_slice(&body)),
    };

    let mut stream = ProcedureStreamResponse {
        code: None,
        stream,
        first: None,
    };
    stream.first = Some(stream.next().await);
    // TODO: Some(poll_fn(|cx| stream.poll_next(cx)).await);

    // TODO: We should poll past the first value to check if it's the only value and set content-type based on it. `ready(...)` will go done straight away.

    (stream.code.unwrap_or(500), stream)
}

pub async fn into_body(
    stream: ProcedureStream,
) -> (u16, impl Stream<Item = Vec<u8>> + Send + 'static) {
    let mut stream = ProcedureStreamResponse {
        code: None,
        stream,
        first: None,
    };
    stream.first = Some(stream.next().await);
    // TODO: Some(poll_fn(|cx| stream.poll_next(cx)).await);

    // TODO: We should poll past the first value to check if it's the only value and set content-type based on it. `ready(...)` will go done straight away.

    (stream.code.unwrap_or(500), stream)
}

// TODO: Sealing fields at least?
struct ProcedureStreamResponse {
    code: Option<u16>,
    first: Option<Option<Vec<u8>>>,
    stream: ProcedureStream,
}

impl Stream for ProcedureStreamResponse {
    type Item = Vec<u8>;

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
                                                           //     .map_err(|err| {
                                                           //     // TODO: Configure handling of this error and how we log it???
                                                           //     serde_json::to_vec(&ProcedureError::Serializer(err.to_string()))
                                                           //         .expect("bruh")
                                                           // })),
                ),
                Some(Err(err)) => (
                    err.status(),
                    Some(serde_json::to_vec(&err).unwrap()), // TODO: Error handling
                ),
                None => (200, None),
            }
        };

        // TODO: Only after first item
        if let Some(buf) = &mut buf {
            buf.extend_from_slice(b"\n\n");
        };

        self.code = Some(code);
        Poll::Ready(buf)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

// TODO
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
