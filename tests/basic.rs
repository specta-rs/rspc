use rspc::{BuildError, BuildResult, Config, Rspc};

// TODO: Test that a stream can't be returned from a query/mutation -> Using trybuild

const R: Rspc<()> = Rspc::new();

#[test]
fn test_router_merging() {
    let r1 = R.router().procedure("a", R.query(|_, _: ()| Ok(())));
    let r2 = R.router().procedure("b", R.query(|_, _: ()| Ok(())));
    let r3 = R
        .router()
        .merge("r1", r1)
        .merge("r2", r2)
        .build(Config::new())
        .unwrap();

    // TODO: Test the thing works
}

#[test]
fn test_invalid_router_merging() {
    let result = R
        .router()
        .procedure("@@@", R.query(|_, _: ()| Ok(())))
        .procedure("demo.2", R.query(|_, _: ()| Ok(())))
        .build(Config::new());

    let errors = match result {
        BuildResult::Err(e) => e,
        BuildResult::Ok(_) => panic!("Expected error"),
    };
    assert_eq!(errors.len(), 2);

    assert_eq!(errors[0].expose(), ("@@@".into(), "a procedure or router name contains the character '@' which is not allowed. Names must be alphanumeric or have '_' or '-'".into()));
    assert_eq!(errors[1].expose(), ("demo.2".into(), "a procedure or router name contains the character '.' which is not allowed. Names must be alphanumeric or have '_' or '-'".into()));
}

// #[test]
// fn test_alpha_api() {
//     let r = R
//         .router()
//         .procedure(
//             "todo",
//             R.with(|mw, ctx| async move { mw.next(ctx) })
//                 .query(|ctx, _: ()| {
//                     println!("TODO: {:?}", ctx);
//                     Ok(())
//                 }),
//         )
//         .procedure(
//             "todo2",
//             R.with(|mw, ctx| async move {
//                 let msg = format!(
//                     "[LOG] req='{:?}' ctx='{:?}' input='{:?}'",
//                     mw.req, ctx, mw.input
//                 );
//                 mw.next(ctx).resp(|result| async move {
//                     println!("{msg} result='{result:?}'");
//                     result
//                 })
//             })
//             .with(|mw, ctx| async move {
//                 let msg = format!(
//                     "[LOG2] req='{:?}' ctx='{:?}' input='{:?}'",
//                     mw.req, ctx, mw.input
//                 );
//                 mw.next(ctx).resp(|result| async move {
//                     println!("{msg} result='{result:?}'");
//                     result
//                 })
//             })
//             .query(|ctx, _: ()| {
//                 println!("TODO: {:?}", ctx);
//                 Ok(())
//             }),
//         )
//         .procedure(
//             "todo3",
//             R.query(|ctx, _: ()| {
//                 println!("TODO: {:?}", ctx);
//                 Ok(())
//             }),
//         )
//         .procedure(
//             "demoSubscriptions",
//             R.subscription(|_ctx, _: ()| {
//                 stream! {
//                     println!("Client subscribed to 'pings'");
//                     for i in 0..5 {
//                         println!("Sending ping {}", i);
//                         yield "ping".to_string();
//                         sleep(Duration::from_secs(1)).await;
//                     }
//                 }
//             }),
//         )
//         .compat();

//     r.export_ts(PathBuf::from("./demo.bindings.ts")).unwrap();
// }

// // TODO: Implement this
// // #[test]
// // fn test_router_merging() {
// //     const R: Rspc = Rspc::new();

// //     let r1 = R.router().procedure("a", R.query(|ctx, _: ()| Ok(())));

// //     let r2 = R.router().procedure("b", R.query(|ctx, _: ()| Ok(())));

// //     let r = R.router().merge("a", r1).merge("b", r2);
// // }

// #[test]
// fn test_context_switching() {
//     const R: Rspc = Rspc::new();

//     let p = R
//         .with(|mw, ctx| async move { mw.next((ctx, 42)) })
//         .with(|mw, ctx| async move { mw.next((ctx, 42)) })
//         .with(|mw, ctx| async move { mw.next(ctx) })
//         .query(|ctx, _: ()| {
//             let ((_, _), _) = ctx; // Assert correct type

//             Ok(())
//         });
// }

// #[test]
// fn test_init_from_function() {
//     const R: Rspc = Rspc::new();

//     fn demo() -> impl ProcedureLike<LayerCtx = ((), i32)> {
//         R.with(|mw, ctx| async move {
//             mw.next((ctx, 42)) // Context switch
//         })
//     }

//     let p = demo().query(|ctx, _: ()| {
//         let (_, _) = ctx; // Assert correct type
//         Ok(())
//     });
// }

// #[test]
// fn with_middleware_from_func() {
//     pub fn library<TLCtx>() -> impl ConstrainedMiddleware<TLCtx, NewCtx = (TLCtx, i32)>
//     where
//         TLCtx: Send + Sync + 'static,
//     {
//         |mw, ctx| async move { mw.next((ctx, 42)) }
//     }

//     let p = R.with(library()).with(library()).query(|ctx, _: ()| {
//         let ((_, _), _) = ctx; // Assert correct type
//         Ok(())
//     });
// }

// #[test]
// fn middleware_args() {
//     pub struct LibraryArgsMap;

//     impl MwArgMapper for LibraryArgsMap {
//         type Input<T> = (T, i32)
//             where
//                 T: DeserializeOwned + Type + 'static;
//         type State = i32;

//         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
//             arg: Self::Input<T>,
//         ) -> (T, Self::State) {
//             (arg.0, arg.1)
//         }
//     }

//     let _p = R
//         .with2(
//             MwArgMapperMiddleware::<LibraryArgsMap>::new().mount(|mw, ctx, state| async move {
//                 let _state: i32 = state; // Assert correct type
//                 let _ctx: () = (); // Assert correct type

//                 mw.next((ctx, 42))
//             }),
//         )
//         .query(|ctx, _: ()| {
//             println!("TODO: {:?}", ctx);
//             let _ = ctx.0; // Test Rust inference is working
//             Ok(())
//         });
// }

// #[test]
// fn multiple_middleware_args() {
//     pub struct DoubleTupleMapper;

//     impl MwArgMapper for DoubleTupleMapper {
//         type Input<T> = (T, Self::State)
//             where
//                 T: DeserializeOwned + Type + 'static;
//         type State = ((), ());

//         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
//             arg: Self::Input<T>,
//         ) -> (T, Self::State) {
//             (arg.0, ((), ()))
//         }
//     }

//     pub struct TripleTupleMapper;

//     impl MwArgMapper for TripleTupleMapper {
//         type Input<T> = (T, Self::State)
//             where
//                 T: DeserializeOwned + Type + 'static;
//         type State = ((), (), ());

//         fn map<T: Serialize + DeserializeOwned + Type + 'static>(
//             arg: Self::Input<T>,
//         ) -> (T, Self::State) {
//             (arg.0, ((), (), ()))
//         }
//     }

//     let p = R
//         .with2(MwArgMapperMiddleware::<DoubleTupleMapper>::new().mount(
//             |mw, ctx, state| async move {
//                 let (_, _) = state; // Assert type is correct
//                 mw.next(ctx)
//             },
//         ))
//         .with2(MwArgMapperMiddleware::<TripleTupleMapper>::new().mount(
//             |mw, ctx, state| async move {
//                 let (_, _, _) = state; // Assert type is correct

//                 mw.next(ctx)
//             },
//         ))
//         .query(|_, _: i32| Ok(()));

//     let _r = R
//         .router()
//         .procedure("demo", p)
//         .compat()
//         .export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./demo2.bindings.ts"))
//         .unwrap();
// }
