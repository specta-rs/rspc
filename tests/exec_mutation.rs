use async_stream::stream;
use rspc::{
    internal::exec::{Executor, Request, ResponseError, TokioRuntime, ValueOrError},
    Config, ExecError, Rspc,
};
use serde_json::Value;

mod utils;
use utils::*;

const R: Rspc<()> = Rspc::new();

#[tokio::test]
async fn test_exec_mutation() {
    let r = R
        .router()
        .procedure("a", R.query(|_, _: ()| ""))
        .procedure("b", R.subscription(|_, _: ()| stream! {}))
        .procedure(
            "c",
            R.mutation(|_, _: ()| {
                atomic_procedure!("c");
                42
            }),
        )
        .procedure(
            "d",
            R.mutation(|_, _: ()| {
                atomic_procedure!("d");
                async move { 43 }
            }),
        )
        .procedure(
            "e",
            R.mutation(|_, input: String| {
                atomic_procedure!("e");
                async move { input }
            }),
        )
        .build(Config::new())
        .unwrap()
        .arced();

    let e = Executor::<_, TokioRuntime>::new(r);

    // Ensure request for mutation doesn't resolve to a query
    assert_resp(
        &e,
        Request::Mutation {
            path: "a".into(),
            input: None,
        },
        ValueOrError::Error(ExecError::OperationNotFound.into()),
    )
    .await;

    // Ensure request for mutation doesn't resolve to a subscription
    assert_resp(
        &e,
        Request::Mutation {
            path: "b".into(),
            input: None,
        },
        ValueOrError::Error(ExecError::OperationNotFound.into()),
    )
    .await;

    // Test some synchronous resolver works
    assert_resp(
        &e,
        Request::Mutation {
            path: "c".into(),
            input: None,
        },
        ValueOrError::Value(42.into()),
    )
    .await;

    // Test some asynchronous resolver works
    assert_resp(
        &e,
        Request::Mutation {
            path: "d".into(),
            input: None,
        },
        ValueOrError::Value(43.into()),
    )
    .await;

    // Test with input
    assert_resp(
        &e,
        Request::Mutation {
            path: "e".into(),
            input: Some(Value::String("hello".into())),
        },
        ValueOrError::Value("hello".into()),
    )
    .await;

    // Test passing no input when procedure expects it
    assert_resp(
        &e,
        Request::Mutation {
            path: "e".into(),
            input: None,
        },
        ValueOrError::Error(ResponseError {
            code: 500,
            message:
                "error deserializing procedure arguments: invalid type: null, expected a string"
                    .into(),
            data: None,
        }),
    )
    .await;

    // Test passing incorrect input
    assert_resp(
        &e,
        Request::Mutation {
            path: "e".into(),
            input: Some(42.into()),
        },
        ValueOrError::Error(ResponseError {
            code: 500,
            message:
                "error deserializing procedure arguments: invalid type: integer `42`, expected a string"
                    .into(),
            data: None,
        }),
    )
    .await;
}
