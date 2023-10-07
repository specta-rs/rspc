use async_stream::stream;
use rspc::RouterBuilder;

use crate::R;

fn mount_inner() -> RouterBuilder<()> {
    R.router().procedure(
        "demo",
        R.query(|_ctx, _: ()| async move { Ok("Hello World") }),
    )
}

fn mount_inner2() -> RouterBuilder<()> {
    R.router().procedure(
        "demo",
        R.query(|_ctx, _: ()| async move { Ok("Hello World") }),
    )
}

fn mount() -> RouterBuilder<()> {
    R.router()
        .procedure(
            "demo",
            R.query(|_ctx, _: ()| async move { Ok("Hello World") }),
        )
        .merge("inner.", mount_inner())
        .merge("inner2.", mount_inner2())
        .procedure(
            "pings",
            R.subscription(|_ctx, _args: ()| {
                stream! {}
            }),
        )
}

#[allow(dead_code)]
fn main() {
    let _r = R.router().merge("java.", mount());

    // TODO: Hookup your router to a webserver like Axum or a Tauri desktop app using the Tauri IPC adapter.
}
