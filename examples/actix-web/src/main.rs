use std::path::PathBuf;

use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use example_core::{mount, Ctx};
use futures::{StreamExt, TryStreamExt};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world from Actix Web!")
}

#[post("/upload")]
async fn upload(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(field)) = payload.try_next().await {
        println!(
            "{:?} {:?} {:?}",
            field.name().map(|v| v.to_string()),
            field.content_type().map(|v| v.to_string()),
            field.collect::<Vec<_>>().await
        );
    }

    HttpResponse::Ok().body("Done!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    // let procedures = rspc_devtools::mount(procedures, &types);

    // TODO: CORS

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{addr}/rspc/version");
    HttpServer::new(move || {
        App::new()
            // Don't use permissive CORS in production!
            .wrap(Cors::permissive())
            .service(hello)
            .service(upload)
        // TODO
        // .service(web::scope("/rspc").configure(
        //     rspc_actix_web::Endpoint::builder(procedures.clone()).build(|| {
        //         // println!("Client requested operation '{}'", parts.uri.path()); // TODO: Fix this
        //         // Ctx {}
        //         todo!();
        //     }),
        // ))
    })
    .bind(addr)?
    .run()
    .await
}
