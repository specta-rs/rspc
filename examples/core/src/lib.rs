use std::{
    marker::PhantomData,
    time::{Instant, SystemTime},
};

use rspc::{
    middleware::Middleware, Procedure, ProcedureBuilder, ResolverInput, ResolverOutput, Router,
};
use rspc_cache::{cache, cache_ttl, CacheState, Memory};
use rspc_invalidation::Invalidate;
use rspc_zer::Zer;
use serde::{Deserialize, Serialize};
use specta::Type;
use thiserror::Error;
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Type)]
pub struct MySession {
    name: String,
}

// `Clone` is only required for usage with Websockets
#[derive(Clone)]
pub struct Ctx {}

#[derive(Serialize, Type)]
pub struct MyCustomType(String);

#[derive(Debug, Deserialize, Type, Validate)]
pub struct ValidatedType {
    #[validate(email)]
    mail: String,
}

#[derive(Type, Serialize)]
#[serde(tag = "type")]
#[specta(export = false)]
pub enum DeserializationError {
    // Is not a map-type so invalid.
    A(String),
}

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "type")]
pub enum Error {
    #[error("you made a mistake: {0}")]
    Mistake(String),
    #[error("validation: {0}")]
    Validator(#[from] rspc_validator::RspcValidatorError),
    #[error("authorization: {0}")]
    Authorization(#[from] rspc_zer::UnauthorizedError), // TODO: This ends up being cringe: `{"type":"Authorization","error":"Unauthorized"}`
    #[error("internal error: {0}")]
    #[serde(skip)]
    InternalError(#[from] anyhow::Error),
}

impl rspc::Error for Error {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        // rspc::ResolverError::new(self.to_string(), Some(self)) // TODO: Typesafe way to achieve this
        rspc::ResolverError::new(
            self,
            None::<std::io::Error>, // TODO: `Some(self)` but `anyhow::Error` is not `Clone`
        )
        .into()
    }
}

pub struct BaseProcedure<TErr = Error>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>(
    ) -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TInput, TResult, TResult>
    where
        TErr: rspc::Error,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure::builder() // You add default middleware here
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

pub fn mount() -> Router<Ctx> {
    Router::new()
        .procedure("withoutBaseProcedure", {
            Procedure::builder::<Error>().query(|ctx: Ctx, id: String| async move { Ok(()) })
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
        // .procedure("sfmPostEdit", {
        //     <BaseProcedure>::builder().query(|ctx, id: String| async move {
        //         println!("UPDATE THE POST {id:?}");
        //         ctx.invalidator.invalidate(InvalidateEvent::Post { id });
        //         Ok(())
        //     })
        // })
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
        // .procedure("manualFlush", {
        //     <BaseProcedure>::builder()
        //         .manual_flush()
        //         .query(|ctx, id: String| async move {
        //             println!("Set cookies");
        //             flush().await;
        //             println!("Do more stuff in background");
        //             Ok(())
        //         })
        // })
        .procedure("validator", {
            <BaseProcedure>::builder()
                .with(rspc_validator::validate())
                .query(|_, input: ValidatedType| async move {
                    println!("{input:?}");
                    Ok(())
                })
        })
        // .procedure("login", {
        //     <BaseProcedure>::builder().query(|ctx, name: String| async move {
        //         ctx.zer.set_session(&MySession { name });
        //         Ok(())
        //     })
        // })
        // .procedure("me", {
        //     <BaseProcedure>::builder().query(|ctx, _: ()| async move { Ok(ctx.zer.session()?) })
        // })
        .procedure("streamInStreamInStreamInStream", {
            // You would never actually do this but it's just checking how the system behaves
            <BaseProcedure>::builder().query(|_, _: ()| async move {
                Ok(rspc::Stream(rspc::Stream(rspc::Stream(
                    futures::stream::once(async move { Ok(42) }),
                ))))
            })
        })

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

pub fn timing_middleware<TError, TCtx, TInput, TResult>(
) -> Middleware<TError, TCtx, TInput, (TResult, String), TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Send + 'static,
    TInput: Send + 'static,
    TResult: Send + Sync + 'static,
{
    Middleware::new(move |ctx: TCtx, input: TInput, next| async move {
        let instant = Instant::now();
        let result = next.exec(ctx, input).await?;
        Ok((result, format!("{:?}", instant.elapsed())))
    })
}
