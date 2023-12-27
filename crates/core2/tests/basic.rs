use rspc_core2::Procedure;

#[tokio::test]
async fn test_procedure() {
    let _p: Procedure = Box::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();

            // Serialize a single value
            ctx.result.serialize(&"todo");
        })
    });

    let _p: Procedure = Box::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();
            // ctx.result.erased_serialize_str("todo");

            // Serialize multiple values
            for i in 0..5 {
                ctx.result.serialize(&i);
            }
        })
    });

    let _p: Procedure = Box::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();

            // TODO: Stream entire file worth of bytes
        })
    });
}

#[tokio::test]
async fn test_executor() {
    // TODO:
    // let executor = Executor::default();
    // executor.
}
