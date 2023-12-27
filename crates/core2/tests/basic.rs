use rspc_core2::Procedure;

#[tokio::test]
async fn test_procedure() {
    let _p = Procedure {
        handler: Box::new(|ctx| {
            Box::pin(async move {
                let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();
                // ctx.result.erased_serialize_str("todo");
                // TODO
            })
        }),
    };

    let _p = Procedure {
        handler: Box::new(|ctx| {
            Box::pin(async move {
                let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();
                // ctx.result.erased_serialize_str("todo");
                // TODO: Stream multiple JSON results
            })
        }),
    };

    let _p = Procedure {
        handler: Box::new(|ctx| {
            Box::pin(async move {
                let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();
                // ctx.result.erased_serialize_str("todo");
                // TODO: Stream entire file worth of bytes
            })
        }),
    };
}

#[tokio::test]
async fn test_executor() {
    // TODO:
    // let executor = Executor::default();
    // executor.
}
