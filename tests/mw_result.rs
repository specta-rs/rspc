// TODO: Fix this
// #[cfg(test)]
// mod tests {
//     use crate::alpha::middleware::AlphaMiddlewareContext;

//     use super::*;

//     fn mw<TMarker, Mw>(m: Mw)
//     where
//         TMarker: Send + 'static,
//         Mw: MwV2<(), TMarker>
//             + Fn(
//                 AlphaMiddlewareContext<
//                     <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
//                 >,
//                 (),
//             ) -> Mw::Fut,
//     {
//     }

//     #[tokio::test]
//     async fn test_mw_results() {
//         // Pass through ctx
//         mw(|mw, ctx| async move { mw.next(ctx) });

//         // Switch ctx
//         mw(|mw, ctx| async move { mw.next(()) });

//         // Handle response
//         mw(|mw, ctx| async move { mw.next(()).resp(|result| async move { result }) });

//         // Middleware args
//         mw(|mw, ctx| async move {
//             let my_mappers_state = mw.state;
//             mw.args::<()>().next(())
//         });

//         // TODO: Handle response returning Result
//         // mw(|mw, ctx| async move { mw.next(()).resp(|result| async move { Ok(result) }) });

//         // TODO: Handle only query/mutation response
//         // mw(|mw, ctx| async move {
//         //     mw.args::<()>().next(()).raw_resp(|resp| {
//         //         match resp {
//         //             ValueOrStream::Value(_) => {},
//         //             ValueOrStream::Stream(_) => {},
//         //         }
//         //     })
//         // });

//         // TODO: Replace stream
//         // mw(|mw, ctx| async move {
//         //     mw.args::<()>().next(()).stream(|stream| {
//         //         async_stream::stream! {
//         //             while let Some(msg) = stream.next().await {
//         //                 yield msg;
//         //             }
//         //         }
//         //     })
//         // });
//     }
// }
