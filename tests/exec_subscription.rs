use async_stream::stream;
use rspc::{
    internal::exec::{Executor, Request, TokioRuntime, ValueOrError},
    Config, ExecError, Rspc,
};

mod utils;
use utils::*;

const R: Rspc<()> = Rspc::new();

// TODO: Test duplicate subscription ID
// TODO: Assert that the subscriptions are being shut down correctly and when expected
// TODO: Test stopping subscriptions
// TODO: Assert the state of the subscription map
// TODO: Assert error when the transport doesn't support subscriptions

#[tokio::test]
async fn test_exec_subscription() {
    let r = R
        .router()
        .procedure("a", R.query(|_, _: ()| ""))
        .procedure("b", R.mutation(|_, _: ()| ""))
        .procedure(
            "c",
            R.subscription(|_, _: ()| {
                stream! {
                    yield 42;
                }
            }),
        )
        .procedure(
            "d",
            R.subscription(|_, _: ()| {
                atomic_procedure!("d");
                async move {
                    stream! {
                        yield 43;
                    }
                }
            }),
        )
        .procedure(
            "e",
            R.subscription(|_, input: String| {
                atomic_procedure!("e");
                async move {
                    stream! { yield input; }
                }
            }),
        )
        .procedure(
            "f",
            R.subscription(|_, _: ()| {
                atomic_procedure!("f");
                async move {
                    stream! {
                        yield 1;
                        yield 2;
                        yield 3;
                    }
                }
            }),
        )
        .build(Config::new())
        .arced();

    let e = Executor::<_, TokioRuntime>::new(r);

    // Ensure request for subscription doesn't resolve to a query
    assert_resp(
        &e,
        Request::Subscription {
            id: "1".into(),
            path: "a".into(),
            input: None,
        },
        ValueOrError::Error(ExecError::OperationNotFound.into()),
    )
    .await;

    // Ensure request for subscription doesn't resolve to a mutation
    assert_resp(
        &e,
        Request::Subscription {
            id: "1".into(),
            path: "b".into(),
            input: None,
        },
        ValueOrError::Error(ExecError::OperationNotFound.into()),
    )
    .await;
}
