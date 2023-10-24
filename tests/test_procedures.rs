use futures::stream::once;
use rspc::Rspc;
use serde::Serialize;
use specta::Type;

#[derive(thiserror::Error, Serialize, Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

// This is purely a test to see if the supported types compile.
#[test]
fn test_procedure_valid_response_types() {
    const R: Rspc<(), Error> = Rspc::new();

    // // TODO: Testing
    // R.router()
    //     .procedure("g", R.query(|_, _: ()| Ok(..0)))
    //     .procedure("g", R.query(|_, _: ()| Ok(once(async move { Ok(..0) }))));

    R.router()
        // // Result Ok
        // .procedure("a", R.query(|_, _: ()| Ok("todo".to_string())))
        // // Result Err
        // .procedure("b", R.query(|_, _: ()| Err::<(), _>(Error("todo"))))
        // Future Result Ok
        .procedure(
            "c",
            R.query(|_, _: ()| async move { Ok("todo".to_string()) }),
        )
        // Future Result Err
        .procedure(
            "d",
            R.query(|_, _: ()| async move { Err::<(), _>(Error("todo")) }),
        )
        // Stream Result Ok
        .procedure(
            "e",
            R.subscription(|_, _: ()| once(async move { Ok("todo".to_string()) })),
        )
        // Stream Result Err
        .procedure(
            "f",
            R.subscription(|_, _: ()| once(async move { Err::<(), _>(Error("todo")) })),
        )
        // Future Stream
        .procedure(
            "i",
            R.subscription(|_, _: ()| async move { once(async move { Ok("todo".to_string()) }) }),
        )
        // Future Stream
        .procedure(
            "j",
            R.subscription(
                |_, _: ()| async move { once(async move { Err::<(), _>(Error("todo")) }) },
            ),
        )
        // Result Stream Ok
        .procedure(
            "g",
            R.subscription(|_, _: ()| Ok(once(async move { Ok("todo".to_string()) }))),
        )
        // Result Stream Err
        .procedure(
            "h",
            R.subscription(|_, _: ()| Ok(once(async move { Err::<(), _>(Error("todo")) }))),
        )
        // Future Result Stream Ok
        .procedure(
            "i",
            R.subscription(
                |_, _: ()| async move { Ok(once(async move { Ok("todo".to_string()) })) },
            ),
        )
        // Future Result Stream Err
        .procedure(
            "j",
            R.subscription(|_, _: ()| async move {
                Ok(once(async move { Err::<(), _>(Error("todo")) }))
            }),
        );
}
