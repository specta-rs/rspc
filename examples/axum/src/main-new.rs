use axum::{
    body::Body,
    extract::{Multipart, Request},
    http::{header, request::Parts, HeaderMap, HeaderName, StatusCode},
    routing::{get, on, post, MethodFilter, MethodRouter},
    Json,
};
use example_core::{mount, Ctx};
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use rspc::{DynOutput, ProcedureError, ProcedureStream, ProcedureStreamMap, Procedures, State};
use rspc_invalidation::Invalidator;
use serde_json::{de::SliceRead, value::RawValue, Value};
use std::{
    convert::Infallible,
    future::{poll_fn, Future},
    marker::PhantomData,
    path::PathBuf,
    pin::{pin, Pin},
    task::{Context, Poll},
};
use streamunordered::{StreamUnordered, StreamYield};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = mount();
    let (procedures, types) = router.build().unwrap();

    rspc::Typescript::default()
        // .formatter(specta_typescript::formatter::prettier),
        .header("// My custom header")
        // .enable_source_maps() // TODO: Fix this
        .export_to(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bindings.ts"),
            &types,
        )
        .unwrap();

    // Be aware this is very experimental and doesn't support many types yet.
    // rspc::Rust::default()
    //     // .header("// My custom header")
    //     .export_to(
    //         PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../client/src/bindings.rs"),
    //         &types,
    //     )
    //     .unwrap();

    // let procedures = rspc_devtools::mount(procedures, &types); // TODO

    // We disable CORS because this is just an example. DON'T DO THIS IN PRODUCTION!
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello 'rspc'!" }))
        // .route(
        //     "/upload",
        //     post(|mut multipart: Multipart| async move {
        //         println!("{:?}", multipart);
        //         while let Some(field) = multipart.next_field().await.unwrap() {
        //             println!(
        //                 "{:?} {:?} {:?}",
        //                 field.name().map(|v| v.to_string()),
        //                 field.content_type().map(|v| v.to_string()),
        //                 field.collect::<Vec<_>>().await
        //             );
        //         }
        //         "Done!"
        //     }),
        // )
        .route(
            "/rspc/custom",
            post(|| async move {
                // println!("{:?}", multipart);

                // while let Some(field) = multipart.next_field().await.unwrap() {
                //     println!(
                //         "{:?} {:?} {:?}",
                //         field.name().map(|v| v.to_string()),
                //         field.content_type().map(|v| v.to_string()),
                //         field.collect::<Vec<_>>().await
                //     );
                // }

                todo!();
            }),
        )
        // .nest(
        //     "/rspc",
        //     rspc_axum::endpoint(procedures, |parts: Parts| {
        //         println!("Client requested operation '{}'", parts.uri.path());
        //         Ctx {}
        //     }),
        // )
        // .nest(
        //     "/rspc",
        //     rspc_axum::Endpoint::builder(procedures).build(|| {
        //         // println!("Client requested operation '{}'", parts.uri.path()); // TODO: Fix this
        //         Ctx {}
        //     }),
        // )
        .nest("/rspc", rspc_handler(procedures))
        .layer(cors);

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{}/rspc/version", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

pub fn rspc_handler(procedures: Procedures<Ctx>) -> axum::Router {
    let mut r = axum::Router::new();
    // TODO: Support file upload and download
    // TODO: `rspc_zer` how worky?

    // for (key, procedure) in procedures.clone() {
    //     r = r.route(
    //         &format!("/{key}"),
    //         on(
    //             MethodFilter::GET.or(MethodFilter::POST),
    //             move |req: rspc_axum::AxumRequest| async move {
    //                 let mut stream = req.deserialize(|buf| {
    //                     let mut input = serde_json::Deserializer::new(SliceRead::new(buf));
    //                     procedure.exec_with_deserializer(Ctx {}, &mut input)
    //                 });
    //                 let (status, is_stream) = stream.next_status().await;

    //                 (
    //                     StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    //                     [
    //                         (
    //                             header::CONTENT_TYPE,
    //                             is_stream
    //                                 .then_some("application/jsonstream")
    //                                 .unwrap_or("application/json"),
    //                         ),
    //                         (HeaderName::from_static("x-rspc"), "1"),
    //                     ],
    //                     Body::from_stream(stream.map(|v| {
    //                         serde_json::to_vec(&v)
    //                             .map_err(|err| err.to_string())
    //                             .map(Ok::<_, Infallible>)
    //                     })),
    //                 )
    //             },
    //         ),
    //     );
    // }

    // TODO: Websocket & batch endpoint

    // // TODO: Supporting zero-flight mutations???
    // // TODO: Streaming back each response separately
    // r.route(
    //     &format!("/~rspc.batch"),
    //     post(move |mut multipart: Multipart| async move {
    //         while let Some(mut field) = multipart.next_field().await.unwrap() {
    //             let name = field.name().unwrap().to_string(); // TODO: Error handling

    //             // field.headers()

    //             // TODO: Don't use `serde_json::Value`
    //             let input: Value = match field.content_type() {
    //                 Some("application/json") => {
    //                     // TODO: Error handling
    //                     serde_json::from_slice(field.bytes().await.unwrap().as_ref()).unwrap()
    //                 }
    //                 Some(_) => todo!(),
    //                 None => todo!(),
    //             };

    //             let procedure = procedures.get(&*name).unwrap();
    //             println!("{:?} {:?} {:?}", name, input, procedure);
    //         }

    //         // TODO: Streaming result & configurable content size
    //         (
    //             [(header::CONTENT_TYPE, "application/jsonstream")],
    //             // Body::from_stream(stream.map(|v| {
    //             //     serde_json::to_vec(&v)
    //             //         .map_err(|err| err.to_string())
    //             //         .map(Ok::<_, Infallible>)
    //             // })),
    //             "Testing",
    //         )
    //     }),
    // )

    // TODO: If Tanstack Query cache key is `input` how does `File` work?

    // TODO: Allowing `GET` requests too?
    // TODO: WebSocket upgrade
    // TODO: Optional zero-flight mutations
    // TODO: Document CDN caching options with this setup
    r.route(
        "/",
        post(move |parts: Parts, mut multipart: Multipart| async move {
            let invalidator = rspc_invalidation::Invalidator::default();
            let (zer, zer_response) = rspc_zer::Zer::from_request(
                "session",
                "some_secret".as_ref(),
                parts.headers.get("cookie"),
            )
            .unwrap(); // TODO: Error handling

            let runtime = Runtime::new(
                procedures.clone(), // TODO: It would be nice if we could avoid clone
                |v| {
                    match v {
                        Ok(v) => serde_json::to_vec(&v.as_serialize().unwrap()),
                        Err(err) => serde_json::to_vec(&err),
                    }
                    .map_err(|err| err.to_string())
                },
                move || Ctx {
                    invalidator: invalidator.clone(),
                    zer: zer.clone(),
                },
            );

            while let Some(field) = multipart.next_field().await.unwrap() {
                let name = field.name().unwrap().to_string(); // TODO: Error handling
                let mut bytes = field.bytes().await.unwrap(); // TODO: Error handling
                runtime.exec(
                    &name,
                    &mut serde_json::Deserializer::new(SliceRead::new(&bytes)),
                );
            }

            runtime.await_flush().await;

            // TODO: Queue SFM's

            // let mut headers = HeaderMap::new();
            // headers.insert(header::CONTENT_TYPE, "text/x-rspc".parse().unwrap());
            // if let Some(h) = zer_response.set_cookie_header() {
            //     headers.insert(header::SET_COOKIE, h.parse().unwrap());
            // }

            // (
            //     headers,
            //     Body::from_stream(Prototype {
            //         runtime,
            //         sfm: false,
            //         invalidator,
            //         ctx,
            //         procedures: procedures.clone(),
            //     }),
            // )

            todo!();
        }),
    )
}

// TODO: Move this into into `rspc`????
pub struct Runtime<TCtx, T, TCtxFn, TSerializeFn> {
    procedures: Procedures<TCtx>,
    runtime: StreamUnordered<ProcedureStreamMap<TSerializeFn, T>>,
    ctx_fn: TCtxFn,
    serialize_fn: TSerializeFn,
}

impl<TCtx, T, TCtxFn, TSerializeFn> Runtime<TCtx, T, TCtxFn, TSerializeFn>
where
    TCtxFn: Fn() -> TCtx,
    // TODO: Error type not as `String`
    TSerializeFn: Fn(Result<DynOutput, ProcedureError>) -> Result<T, String>,
{
    pub fn new(procedures: Procedures<TCtx>, serializer: TSerializeFn, ctx_fn: TCtxFn) -> Self {
        todo!();
    }

    // TODO: Allowing non-serde inputs
    // TODO: Taking `serialize_fn` as an input
    pub fn exec<'de>(&mut self, name: &str, input: impl serde::Deserializer<'de> + Send) {
        let procedure = self.procedures.get(&*name).unwrap(); // TODO: Error handling
        let stream = procedure.exec_with_deserializer((self.ctx_fn)(), input);

        self.runtime
            .insert(stream.require_manual_stream().map::<TSerializeFn, T>({
                let a = &self.serialize_fn;
                |v| (a)(v)
            }));
    }

    pub async fn await_flush(&self) {
        // poll_fn(|cx| match runtime.poll_next_unpin(cx) {
        //     // `ProcedureStream::require_manual_stream` is set.
        //     Poll::Ready(_) => unreachable!(),
        //     Poll::Pending => {
        //         // Once we know all futures are ready,
        //         // we allow them all to flush.
        //         if runtime
        //             .iter_mut()
        //             // TODO: If we want to allow the user to opt-in to manually flush and flush within a stream this won't work.
        //             .all(|stream| stream.resolved() || stream.flushable())
        //         {
        //             runtime.iter_mut().for_each(|s| s.stream());
        //             Poll::Ready(())
        //         } else {
        //             Poll::Pending
        //         }
        //     }
        // })
        // .await;

        todo!();
    }
}

impl<TCtx, T, TCtxFn, TSerializeFn> Stream for Runtime<TCtx, T, TCtxFn, TSerializeFn> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

// TODO: This abstraction is soooo bad.
pub struct Prototype<TCtx, E> {
    runtime: StreamUnordered<
        ProcedureStreamMap<
            fn(Result<DynOutput, ProcedureError>) -> Result<Vec<u8>, String>,
            Vec<u8>,
        >,
    >,
    invalidator: Invalidator<E>,
    ctx: TCtx,
    sfm: bool,
    procedures: Procedures<TCtx>,
}

// impl<E> Prototype<E> {}

// TODO: Drop `Unpin` requirement
impl<TCtx: Unpin + Clone + 'static, E: 'static> Stream for Prototype<TCtx, E> {
    type Item = Result<Vec<u8>, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // We spawn SFM's after all futures are resolved, as we don't allow `.invalidate` calls after this point.
            // In general you shouldn't be batching mutations so this won't make a difference to performance.
            if !self.sfm
                && self
                    .as_mut()
                    .get_mut()
                    .runtime
                    .iter_mut()
                    .all(|s| s.resolved())
            {
                self.sfm = true;

                for stream in
                    rspc_invalidation::queue(&self.invalidator, self.ctx.clone(), &self.procedures)
                {
                    self.runtime.insert(
                        stream.map::<fn(Result<DynOutput, ProcedureError>) -> Result<Vec<u8>, String>, Vec<u8>>(|v| {
                            match v {
                                Ok(v) => serde_json::to_vec(&v.as_serialize().unwrap()),
                                Err(err) => serde_json::to_vec(&err),
                            }.map_err(|err| err.to_string())
                        }),
                    );
                }
            }

            let Poll::Ready(v) = self.runtime.poll_next_unpin(cx) else {
                return Poll::Pending;
            };

            return Poll::Ready(match v {
                Some((v, _)) => match v {
                    StreamYield::Item(mut v) => {
                        let id = 0; // TODO: Include identifier to request/query
                        let identifier = 'O' as u8; // TODO: error, oneshot, event or complete message
                                                    // let content_type = ""; // TODO: Include content-type of it
                        let mut buf = vec![id, identifier];
                        buf.append(&mut v);
                        buf.extend_from_slice(b"\n\n");
                        Some(Ok(buf))
                    }
                    StreamYield::Finished(finished_stream) => {
                        // TODO: Complete messages (unless oneshot maybe)
                        finished_stream.remove(Pin::new(&mut self.runtime));
                        continue;
                    }
                },
                None => None,
            });
        }
    }
}

fn encode_msg(a: (), b: (), c: ()) {
    todo!();
}

// TODO: support `GET`
// r = r.route(
//     &format!("/{key}"),
//     // TODO: The json decoding is also way less efficent (`serde_json::Value` as intermediate step)
//     post(move |json: Option<Json<serde_json::Value>>| async move {
//         let mut stream =
//             procedure.exec_with_deserializer(Ctx {}, json.map(|v| v.0).unwrap_or_default());
//         let (status, is_stream) = stream.next_status().await;

//         (
//             StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
//             [
//                 (
//                     header::CONTENT_TYPE,
//                     is_stream
//                         .then_some("application/jsonstream")
//                         .unwrap_or("application/json"),
//                 ),
//                 (HeaderName::from_static("x-rspc"), "1"),
//             ],
//             Body::from_stream(stream.map(|v| {
//                 serde_json::to_vec(&v)
//                     .map_err(|err| err.to_string())
//                     .map(Ok::<_, Infallible>)
//             })),
//         )
//     }),
// );

// r = r.route(
//     &format!("/{key}"),
//     // TODO: The `Json` decoding won't return an rspc errors.
//     // TODO: The json decoding is also way less efficent (`serde_json::Value` as intermediate step)
//     on(
//         MethodFilter::GET.or(MethodFilter::POST),
//         move |input: rspc_axum::AxumRequest| async move {
//             let todo = input.execute(
//                 &procedure,
//                 |buf| &mut serde_json::Deserializer::new(SliceRead::new(buf)),
//                 Ctx {},
//             );
//         },
//     ),
// );

// r = r.route(
//     &format!("/{key}"),
//     post(move |req: Request| async move {
//         // TODO

//         "todo"
//     }),
// );

// let (status, mut stream) =
//     rspc_http::into_body(procedure.exec_with_deserializer(Ctx {}, json.0)).await;
