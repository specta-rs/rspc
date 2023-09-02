use rspc::Router;

use crate::R;

// We merge this router into the main router in `main.rs`.
// This router shows how to do basic queries and mutations and how they tak
pub fn mount() -> Router<()> {
    R.router()
        .procedure("version", R.query(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .procedure("echo", R.query(|_, v: String| v))
        .procedure("echoAsync", R.query(|_, _: i32| async move { 42 }))
        // .procedure(
        //     "error",
        //     R.query(|_, _: ()| {
        //         Err(rspc::Error::new(
        //             rspc::ErrorCode::InternalServerError,
        //             "Something went wrong".into(),
        //         )) as Result<String, rspc::Error>
        //     }),
        // )
        .procedure(
            "transformMe",
            R.query(|_, _: ()| "Hello, world!".to_string()),
        )
        .procedure(
            "sendMsg",
            R.mutation(|_, v: String| {
                println!("Client said '{}'", v);
                v
            }),
        )
}
