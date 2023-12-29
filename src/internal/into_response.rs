//! This file contains the traits used to normalised all valid return types for a resolver into a single normalised `Stream` type.
//!
//! By normalising the types we can constrain them at a higher level and Rust is about to understand the users intentions better, providing some really nice error messages.
//! Instead of the error being "doesn't impl `IntoResponse`", it can be "`specta::Type` must be implemented for X" which is much more helpful.
//!
//! This module is sealed so although it contains public types they will not end up in the public API.

use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
};

use futures::{
    future::Either,
    stream::{once, FlatMap, Flatten, Once},
    Stream, StreamExt,
};

pub struct QueryOrMutation<M>(PhantomData<M>);
pub struct Subscription<M>(PhantomData<M>);

#[cfg(test)]
#[derive(thiserror::Error, serde::Serialize, specta::Type, Debug)]
#[error("{0}")]
struct Error(&'static str);

#[cfg(test)]
const R: crate::Rspc<(), Error> = crate::Rspc::new();

/// `IntoResponse` will transform a specific response type into a normalised response type for a `query` or `mutation`.
pub trait IntoResolverResponse<'a, M> {
    type Stream: Stream<Item = Result<Self::Ok, Self::Err>> + Send + 'a;
    type Ok;
    type Err;

    fn to_stream(self) -> Self::Stream;
}

const _: () = {
    pub enum Marker {}
    impl<'a, T, TErr> IntoResolverResponse<'a, QueryOrMutation<Marker>> for Result<T, TErr>
    where
        T: Send + 'static,
        TErr: Send + 'static,
    {
        type Stream = Once<Ready<Result<T, TErr>>>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            once(ready(self))
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Result Ok
            .procedure("ok", R.query(|_, _: ()| Ok("todo".to_string())))
            // Result Err
            .procedure("err", R.query(|_, _: ()| Err::<(), _>(Error("todo"))));
    }
};

const _: () = {
    pub enum Marker {}
    impl<'a, T, TErr, F> IntoResolverResponse<'a, QueryOrMutation<Marker>> for F
    where
        F: Future<Output = Result<T, TErr>> + Send + 'a,
    {
        type Stream = Once<F>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            once(self)
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Future Result Ok
            .procedure(
                "ok",
                R.query(|_, _: ()| async move { Ok("todo".to_string()) }),
            )
            // Future Result Err
            .procedure(
                "err",
                R.query(|_, _: ()| async move { Err::<(), _>(Error("todo")) }),
            );
    }
};

const _: () = {
    pub enum Marker {}
    impl<'a, T, TErr, S: Stream<Item = Result<T, TErr>> + Send + 'a>
        IntoResolverResponse<'a, Subscription<Marker>> for S
    {
        type Stream = S;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            self
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Stream Result Ok
            .procedure(
                "ok",
                R.subscription(|_, _: ()| once(async move { Ok("todo".to_string()) })),
            )
            // Stream Result Err
            .procedure(
                "err",
                R.subscription(|_, _: ()| once(async move { Err::<(), _>(Error("todo")) })),
            );
    }
};

type StreamOrError<T, TErr, S> = Either<S, Once<Ready<Result<T, TErr>>>>;

const _: () = {
    pub enum Marker {}
    impl<'a, T: Send + 'a, S: Stream<Item = Result<T, TErr>> + Send + 'a, TErr: Send + 'a>
        IntoResolverResponse<'a, Subscription<Marker>> for Result<S, TErr>
    {
        type Stream = StreamOrError<T, TErr, S>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            match self {
                Ok(stream) => stream.left_stream(),
                Err(err) => once(ready(Err(err))).right_stream(),
            }
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Future Stream Ok
            .procedure(
                "ok",
                R.subscription(
                    |_, _: ()| async move { once(async move { Ok("todo".to_string()) }) },
                ),
            )
            // Future Stream Err
            .procedure(
                "err",
                R.subscription(|_, _: ()| async move {
                    once(async move { Err::<(), _>(Error("todo")) })
                }),
            );
    }
};

const _: () = {
    pub enum Marker {}
    impl<
            'a,
            T,
            TErr,
            S: Stream<Item = Result<T, TErr>> + Send,
            F: Future<Output = S> + Send + 'a,
        > IntoResolverResponse<'a, Subscription<Marker>> for F
    {
        type Stream = Flatten<Once<F>>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            once(self).flatten()
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Result Stream Ok
            .procedure(
                "ok",
                R.subscription(|_, _: ()| Ok(once(async move { Ok("todo".to_string()) }))),
            )
            // Result Stream Err
            .procedure(
                "err",
                R.subscription(|_, _: ()| Ok(once(async move { Err::<(), _>(Error("todo")) }))),
            );
    }
};

const _: () = {
    pub enum Marker {}

    impl<
            'a,
            T: Send + 'a,
            TErr: Send + 'a,
            S: Stream<Item = Result<T, TErr>> + Send + 'a,
            F: Future<Output = Result<S, TErr>> + Send + 'a,
        > IntoResolverResponse<'a, Subscription<Marker>> for F
    {
        type Stream = FlatMap<
            Once<F>,
            StreamOrError<T, TErr, S>,
            fn(Result<S, TErr>) -> StreamOrError<T, TErr, S>,
        >;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            once(self).flat_map(|result| match result {
                Ok(s) => s.left_stream(),
                Err(e) => once(ready(Err(e))).right_stream(),
            })
        }
    }

    #[cfg(test)]
    fn test() {
        R.router()
            // Future Result Stream Ok
            .procedure(
                "ok",
                R.subscription(|_, _: ()| async move {
                    Ok(once(async move { Ok("todo".to_string()) }))
                }),
            )
            // Future Result Stream Err
            .procedure(
                "err",
                R.subscription(|_, _: ()| async move {
                    Ok(once(async move { Err::<(), _>(Error("todo")) }))
                }),
            );
    }
};
