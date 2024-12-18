use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use example_core::{create_router, Ctx};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world from Actix Web!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    // let procedures = rspc_devtools::mount(procedures, &types);

    // TODO: CORS

    let addr = "[::]:4000".parse::<std::net::SocketAddr>().unwrap(); // This listens on IPv6 and IPv4
    println!("listening on http://{addr}/rspc/version");
    HttpServer::new(move || {
        App::new()
            // Don't use permissive CORS in production!
            .wrap(Cors::permissive())
            .service(hello)
            .service(web::scope("/rspc").configure(
                rspc_actix_web::Endpoint::builder(procedures.clone()).build(|| {
                    // println!("Client requested operation '{}'", parts.uri.path()); // TODO: Fix this
                    Ctx {}
                }),
            ))
    })
    .bind(addr)?
    .run()
    .await
}
