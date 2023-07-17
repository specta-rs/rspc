// #![allow(dead_code)]

// use rspc::internal::exec::{
//     Executor, ExecutorResult, NoOpSubscriptionManager, Request, Response, TokioRuntime,
//     ValueOrError,
// };

// pub async fn exec(executor: &Executor<(), TokioRuntime>, req: Request) -> Option<Response> {
//     match executor.execute((), req, &mut (None as Option<NoOpSubscriptionManager>)) {
//         ExecutorResult::FutureResponse(fut) => Some(fut.await),
//         ExecutorResult::Response(r) => Some(r),
//         ExecutorResult::None => None,
//     }
// }

// pub async fn assert_resp(e: &Executor<(), TokioRuntime>, req: Request, result: ValueOrError) {
//     let path = match &req {
//         Request::Query { path, .. } => path,
//         Request::Mutation { path, .. } => path,
//         Request::Subscription { path, .. } => path,
//         Request::SubscriptionStop { .. } => unreachable!(),
//     }
//     .clone();
//     assert_eq!(
//         exec(e, req).await,
//         Some(Response::Response { path, result })
//     );
// }

// // atomic_procedure makes sure the procedure is only invoked once
// #[macro_export]
// macro_rules! atomic_procedure {
//     ($name:expr) => {
//         static CALL_COUNT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
//         if CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst) != 0 {
//             panic!("procedure '{}' was invoked more than once!", $name);
//         }
//     };
// }
