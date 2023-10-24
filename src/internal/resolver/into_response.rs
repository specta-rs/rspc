//! This file contains the traits used to normalised all valid return types for a resolver into a single normalised `Stream` type.
//!
//! By normalising the types we can constrain them at a higher level and Rust is about to understand the users intentions better, providing some really nice error messages.
//! Instead of the error being "doesn't impl `IntoResponse`", it can be "`specta::Type` must be implemented for X" which is much more helpful.
//!
//! This module is sealed so although it contains public types they will not end up in the public API.

use std::future::{ready, Future, Ready};

use futures::{
    future::Either,
    stream::{once, FlatMap, Flatten, Once},
    Stream, StreamExt,
};

use crate::internal::resolver::{QueryOrMutation, Subscription};

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
};

const _: () = {
    pub enum Marker {}
    impl<'a, T: Send + 'a, S: Stream<Item = Result<T, TErr>> + Send + 'a, TErr: Send + 'a>
        IntoResolverResponse<'a, Subscription<Marker>> for Result<S, TErr>
    {
        type Stream = Either<S, Once<Ready<Result<T, TErr>>>>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            match self {
                Ok(stream) => stream.left_stream(),
                Err(err) => once(ready(Err(err))).right_stream(),
            }
        }
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
};

const _: () = {
    pub enum Marker {}
    type Inner<T, TErr, S> = Either<S, Once<Ready<Result<T, TErr>>>>;

    impl<
            'a,
            T: Send + 'a,
            TErr: Send + 'a,
            S: Stream<Item = Result<T, TErr>> + Send + 'a,
            F: Future<Output = Result<S, TErr>> + Send + 'a,
        > IntoResolverResponse<'a, Subscription<Marker>> for F
    {
        type Stream = FlatMap<Once<F>, Inner<T, TErr, S>, fn(Result<S, TErr>) -> Inner<T, TErr, S>>;
        type Ok = T;
        type Err = TErr;

        fn to_stream(self) -> Self::Stream {
            once(self).flat_map(|result| match result {
                Ok(s) => s.left_stream(),
                Err(e) => once(ready(Err(e))).right_stream(),
            })
        }
    }
};
