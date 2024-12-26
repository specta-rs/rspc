use axum::{
    body::Body,
    extract::Multipart,
    http::{header, request::Parts, HeaderMap},
    routing::{get, post},
};
use example_core::{mount, Ctx};
use futures::{Stream, StreamExt};
use rspc::{DynOutput, ProcedureError, ProcedureStream, ProcedureStreamMap, Procedures};
use rspc_invalidation::Invalidator;
use serde_json::Value;
use std::{
    convert::Infallible,
    future::poll_fn,
    path::PathBuf,
    pin::Pin,
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
        .route("/", get(|| async { "rspc 🤝 Axum!" }))
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
    let r = axum::Router::new();

    // TODO: Support file upload and download

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
            let ctx = Ctx {
                invalidator: invalidator.clone(),
                zer,
            };

            let mut runtime = StreamUnordered::new();
            // TODO: Move onto `Prototype`???
            let spawn = |runtime: &mut StreamUnordered<_>, p: ProcedureStream| {
                runtime.insert(p.require_manual_stream().map::<fn(
                    Result<DynOutput, ProcedureError>,
                )
                    -> Result<Vec<u8>, String>, Vec<u8>>(
                    |v| {
                        match v {
                            Ok(v) => serde_json::to_vec(&v.as_serialize().unwrap()),
                            Err(err) => serde_json::to_vec(&err),
                        }
                        .map_err(|err| err.to_string())
                    },
                ));
            };

            // TODO: If a file was being uploaded this would require reading the whole body until the `runtime` is polled.
            while let Some(field) = multipart.next_field().await.unwrap() {
                let name = field.name().unwrap().to_string(); // TODO: Error handling

                let procedure = procedures.get(&*name).unwrap();

                // TODO: Error handling
                spawn(
                    &mut runtime,
                    match field.content_type() {
                        // Some("text/x-binario") => procedure.exec_with_value(
                        //     ctx.clone(),
                        //     // TODO: Stream decoding is pretty rough with multipart so we omit it for now.
                        //     rspc_binario::BinarioStream(field.bytes().await.unwrap().to_vec()),
                        // ),
                        _ => procedure.exec_with_deserializer(
                            ctx.clone(),
                            // TODO: Don't use `serde_json::Value`
                            serde_json::from_slice::<Value>(field.bytes().await.unwrap().as_ref())
                                .unwrap(),
                        ),
                    },
                )
            }

            // TODO: Move onto `Prototype`???
            poll_fn(|cx| match runtime.poll_next_unpin(cx) {
                // `ProcedureStream::require_manual_stream` is set.
                Poll::Ready(_) => unreachable!(),
                Poll::Pending => {
                    // Once we know all futures are ready,
                    // we allow them all to flush.
                    if runtime
                        .iter_mut()
                        // TODO: If we want to allow the user to opt-in to manually flush and flush within a stream this won't work.
                        .all(|stream| stream.resolved() || stream.flushable())
                    {
                        runtime.iter_mut().for_each(|s| s.stream());
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                }
            })
            .await;

            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "text/x-rspc".parse().unwrap());
            if let Some(h) = zer_response.set_cookie_header() {
                headers.insert(header::SET_COOKIE, h.parse().unwrap());
            }

            (
                headers,
                Body::from_stream(Prototype {
                    runtime,
                    sfm: false,
                    invalidator,
                    ctx,
                    procedures: procedures.clone(),
                }),
            )
        }),
    )
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

// fn encode_msg(a: (), b: (), c: ()) {
//     todo!();
// }

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
