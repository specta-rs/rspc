use async_stream::stream;
use rspc::{Router, RouterBuilderLike};

fn mount_inner() -> impl RouterBuilderLike<()> {
    Router::new().query("demo", |t| t(|_ctx, _: ()| async move { "Hello World" }))
}

fn mount() -> impl RouterBuilderLike<()> {
    Router::<()>::new()
        .query("demo", |t| t(|_ctx, _: ()| async move { "Hello World" }))
        .merge("inner.", mount_inner())
        .subscription("pings", |t| {
            t(|_ctx, _args: ()| {
                stream! {}
            })
        })
}

fn main() {
    let _r = Router::<()>::new().merge("java.", mount());

    // TODO: Hookup your router to a webserver like Axum or a Tauri desktop app using the Tauri IPC adapter.
}
