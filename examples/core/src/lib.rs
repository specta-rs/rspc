use std::{marker::PhantomData, time::SystemTime};

use async_stream::stream;
use rspc::{
    middleware::Middleware, Error2, Extension, Procedure2, ProcedureBuilder, ResolverInput,
    ResolverOutput, Router2,
};
use rspc_cache::{cache, cache_ttl, CacheState, Memory};
use rspc_invalidation::Invalidate;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tracing::info;

// `Clone` is only required for usage with Websockets
#[derive(Clone)]
pub struct Ctx {
    pub invalidator: Invalidator,
}

#[derive(Serialize, Type)]
pub struct MyCustomType(String);

#[derive(Type, Serialize)]
#[serde(tag = "type")]
#[specta(export = false)]
pub enum DeserializationError {
    // Is not a map-type so invalid.
    A(String),
}

// http://[::]:4000/rspc/version
// http://[::]:4000/legacy/version

// http://[::]:4000/rspc/nested.hello
// http://[::]:4000/legacy/nested.hello

// http://[::]:4000/rspc/error
// http://[::]:4000/legacy/error

// http://[::]:4000/rspc/echo
// http://[::]:4000/legacy/echo

// http://[::]:4000/rspc/echo?input=42
// http://[::]:4000/legacy/echo?input=42

fn mount() -> rspc::Router<Ctx> {
    let inner = rspc::Router::<Ctx>::new().query("hello", |t| t(|_, _: ()| "Hello World!"));

    let router = rspc::Router::<Ctx>::new()
        .merge("nested.", inner)
        .query("version", |t| {
            t(|_, _: ()| {
                info!("Hello World from Version Query!");

                env!("CARGO_PKG_VERSION")
            })
        })
        .query("panic", |t| t(|_, _: ()| todo!()))
        // .mutation("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
        .query("echo", |t| t(|_, v: String| v))
        .query("error", |t| {
            t(|_, _: ()| {
                Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Something went wrong".into(),
                )) as Result<String, rspc::Error>
            })
        })
        .query("transformMe", |t| t(|_, _: ()| "Hello, world!".to_string()))
        .mutation("sendMsg", |t| {
            t(|_, v: String| {
                println!("Client said '{}'", v);
                v
            })
        })
        // .mutation("anotherOne", |t| t(|_, v: String| Ok(MyCustomType(v))))
        .subscription("pings", |t| {
            t(|_ctx, _args: ()| {
                stream! {
                    println!("Client subscribed to 'pings'");
                    for i in 0..5 {
                        println!("Sending ping {}", i);
                        yield "ping".to_string();
                        // sleep(Duration::from_secs(1)).await; // TODO: Figure this out. Async runtime is now not determined so maybe inject.
                    }
                }
            })
        })
        // TODO: Results being returned from subscriptions
        // .subscription("errorPings", |t| t(|_ctx, _args: ()| {
        //     stream! {
        //         for i in 0..5 {
        //             yield Ok("ping".to_string());
        //             sleep(Duration::from_secs(1)).await;
        //         }
        //         yield Err(rspc::Error::new(ErrorCode::InternalServerError, "Something went wrong".into()));
        //     }
        // }))
        .build();

    router
}

#[derive(Debug, Error, Serialize, Type)]
pub enum Error {
    #[error("you made a mistake: {0}")]
    Mistake(String),
}

impl Error2 for Error {
    fn into_resolver_error(self) -> rspc::ResolverError {
        rspc::ResolverError::new(self.to_string(), None::<std::io::Error>)
    }
}

pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>(
    ) -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TInput, TResult, TResult>
    where
        TErr: Error2,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure2::builder() // You add default middleware here
    }
}

#[derive(Type)]
struct SerialisationError;
impl Serialize for SerialisationError {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        Err(S::Error::custom("lol"))
    }
}

fn test_unstable_stuff(router: Router2<Ctx>) -> Router2<Ctx> {
    router
        .procedure("withoutBaseProcedure", {
            Procedure2::builder::<Error>().query(|ctx: Ctx, id: String| async move { Ok(()) })
        })
        .procedure("newstuff", {
            <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
        .procedure("newstuff2", {
            <BaseProcedure>::builder()
                // .with(invalidation(|ctx: Ctx, key, event| false))
                .with(Middleware::new(
                    move |ctx: Ctx, input: (), next| async move {
                        let result = next.exec(ctx, input).await;
                        result
                    },
                ))
                .query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
        })
        .procedure("newstuffpanic", {
            <BaseProcedure>::builder().query(|_, _: ()| async move { Ok(todo!()) })
        })
        .procedure("newstuffser", {
            <BaseProcedure>::builder().query(|_, _: ()| async move { Ok(SerialisationError) })
        })
        .setup(CacheState::builder(Memory::new()).mount())
        .procedure("cached", {
            <BaseProcedure>::builder()
                .with(cache())
                .query(|_, _: ()| async {
                    // if input.some_arg {}
                    cache_ttl(10);

                    Ok(SystemTime::now())
                })
        })
        .procedure("sfmPost", {
            <BaseProcedure>::builder()
                .with(Middleware::new(
                    move |ctx: Ctx, input: (String, ()), next| async move {
                        let result = next.exec(ctx, input.0).await;
                        result
                    },
                ))
                .with(Invalidator::with(|event| {
                    println!("--- BEFORE");
                    if let InvalidateEvent::Post { id } = event {
                        return Invalidate::One((id.to_string(), ()));
                    }
                    Invalidate::None
                }))
                .query(|_, id: String| async {
                    println!("FETCH POST FROM DB");
                    Ok(id)
                })
            // .with(Invalidator::with(|event| {
            //     println!("--- AFTER");
            //     if let InvalidateEvent::Post { id } = event {
            //         return Invalidate::One((id.to_string(), ()));
            //     }
            //     Invalidate::None
            // }))
        })
        .procedure("sfmPostEdit", {
            <BaseProcedure>::builder().query(|ctx, id: String| async move {
                println!("UPDATE THE POST {id:?}");
                ctx.invalidator.invalidate(InvalidateEvent::Post { id });
                Ok(())
            })
        })

    // .procedure("sfmStatefulPost", {
    //     <BaseProcedure>::builder()
    //         // .with(Invalidator::mw(|ctx, input, event| {
    //         //     event == InvalidateEvent::InvalidateKey(input.id)
    //         // }))
    //         .query(|_, id: String| async {
    //             // Fetch the post from the DB
    //             Ok(id)
    //         })
    // })
    // .procedure("fileupload", {
    //     <BaseProcedure>::builder().query(|_, _: File| async { Ok(env!("CARGO_PKG_VERSION")) })
    // })
}

// .with(Invalidator::mw(|ctx, input, event| {
//     event == InvalidateEvent::InvalidateKey("abc".into())
// }))
// .with(Invalidator::mw_with_result(|ctx, input, result, event| {
//     event == InvalidateEvent::InvalidateKey("abc".into())
// }))

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub enum InvalidateEvent {
    Post { id: String },
    InvalidateKey(String),
}
pub type Invalidator = rspc_invalidation::Invalidator<InvalidateEvent>;

// TODO: Debug, etc
pub struct File<T = ()>(T);

pub fn create_router() -> Router2<Ctx> {
    let router = Router2::from(mount());
    let router = test_unstable_stuff(router);

    router
}
