use axum::{
    body::Body,
    extract::{Multipart, Request},
    http::{header, HeaderName, StatusCode},
    routing::{get, on, post, MethodFilter, MethodRouter},
    Json,
};
use example_core::{create_router, Ctx};
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use rspc::{ProcedureStreamValue, Procedures, State};
use serde_json::{de::SliceRead, value::RawValue, Value};
use std::{
    convert::Infallible,
    future::Future,
    path::PathBuf,
    pin::{pin, Pin},
    task::{Context, Poll},
};
use streamunordered::{StreamUnordered, StreamYield};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let router = create_router();
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
        .route(
            "/upload",
            post(|mut multipart: Multipart| async move {
                println!("{:?}", multipart);

                while let Some(field) = multipart.next_field().await.unwrap() {
                    println!(
                        "{:?} {:?} {:?}",
                        field.name().map(|v| v.to_string()),
                        field.content_type().map(|v| v.to_string()),
                        field.collect::<Vec<_>>().await
                    );
                }

                "Done!"
            }),
        )
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
        post(move |mut multipart: Multipart| async move {
            let mut runtime = StreamUnordered::new();

            let invalidator = rspc_invalidation::Invalidator::default();
            let ctx = Ctx {
                invalidator: invalidator.clone(),
            };

            // TODO: If a file was being uploaded this would require reading the whole body until the `runtime` is polled.
            while let Some(field) = multipart.next_field().await.unwrap() {
                let name = field.name().unwrap().to_string(); // TODO: Error handling

                // field.headers()

                // TODO: Don't use `serde_json::Value`
                let input: Value = match field.content_type() {
                    // TODO:
                    // Some("application/json") => {
                    //     // TODO: Error handling
                    //     serde_json::from_slice(field.bytes().await.unwrap().as_ref()).unwrap()
                    // }
                    // Some(_) => todo!(),
                    // None => todo!(),
                    _ => serde_json::from_slice(field.bytes().await.unwrap().as_ref()).unwrap(),
                };

                let procedure = procedures.get(&*name).unwrap();
                println!("{:?} {:?} {:?}", name, input, procedure);

                let stream = procedure.exec_with_deserializer(ctx.clone(), input);

                runtime.insert(stream.map::<fn(ProcedureStreamValue) -> _, Vec<u8>>(|v| {
                    serde_json::to_vec(&v).map_err(|err| err.to_string())
                }));

                // TODO: Spawn onto runtime
                // let (status, is_stream) = stream.next_status().await;

                // println!("{:?} {:?}", status, is_stream);

                // (
                //     StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                //     [
                //         (
                //             header::CONTENT_TYPE,
                //             is_stream
                //                 .then_some("application/jsonstream")
                //                 .unwrap_or("application/json"),
                //         ),
                //         (HeaderName::from_static("x-rspc"), "1"),
                //     ],
                //     Body::from_stream(stream.map(|v| {
                //         serde_json::to_vec(&v)
                //             .map_err(|err| err.to_string())
                //             .map(Ok::<_, Infallible>)
                //     })),
                // )
            }

            // TODO: Wait until the full stream is Mattrax-style flushed to run this.
            let fut = tokio::time::sleep(std::time::Duration::from_secs(1));
            tokio::select! {
                _ = runtime.next() => {}
                _ = fut => {}
            }

            for stream in rspc_invalidation::queue(&invalidator, || ctx.clone(), &procedures) {
                runtime.insert(stream.map::<fn(ProcedureStreamValue) -> _, Vec<u8>>(|v| {
                    serde_json::to_vec(&v).map_err(|err| err.to_string())
                }));
            }

            (
                [(header::CONTENT_TYPE, "text/x-rspc")],
                Body::from_stream(Prototype { runtime }),
            )
        }),
    )
}

pub struct Prototype<S: Stream<Item = Vec<u8>>> {
    runtime: StreamUnordered<S>,
}

// TODO: Should `S: 'static` be a thing?
impl<S: Stream<Item = Vec<u8>> + 'static> Stream for Prototype<S> {
    type Item = Result<Vec<u8>, Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Poll::Ready(v) = self.runtime.poll_next_unpin(cx) else {
                return Poll::Pending;
            };

            return Poll::Ready(match v {
                Some((v, i)) => match v {
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
