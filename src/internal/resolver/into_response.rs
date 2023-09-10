//! This file contains the traits used to normalised all valid return types for a resolver into a single normalised `Stream` type.
//!
//! By normalising the types we can constrain them at a higher level and Rust is about to understand the users intentions better, providing some really nice error messages.
//! Instead of the error being "doesn't impl `IntoResponse`", it can be "`specta::Type` must be implemented for X" which is much more helpful.
//!
//! This module is sealed so although it contains public types they will not end up in the public API.

use std::future::{ready, Future, Ready};

use futures::{
    stream::{once, Once},
    Stream,
};

/// `IntoResponse` will transform a specific response type into a normalised response type for a `query` or `mutation`.
pub trait IntoQueryMutationResponse<'a, M, TErr> {
    type Stream: Stream<Item = Result<Self::Ok, TErr>> + Send + 'a;
    type Ok;

    fn to_stream(self) -> Self::Stream;
}

const _: () = {
    pub enum Marker {}
    impl<'a, T, TErr> IntoQueryMutationResponse<'a, Marker, TErr> for Result<T, TErr>
    where
        T: Send + 'static,
        TErr: Send + 'static,
    {
        type Stream = Once<Ready<Result<T, TErr>>>;
        type Ok = T;

        fn to_stream(self) -> Self::Stream {
            once(ready(self))
        }
    }
};

const _: () = {
    pub enum Marker {}
    impl<'a, T, TErr, F> IntoQueryMutationResponse<'a, Marker, TErr> for F
    where
        F: Future<Output = Result<T, TErr>> + Send + 'a,
    {
        type Stream = Once<F>;
        type Ok = T;

        fn to_stream(self) -> Self::Stream {
            once(self)
        }
    }
};

// TODO: Copy the above type once stable
/// `IntoResponse` will transform a specific response type into a normalised response type for a `subscription`.
///
/// This type primarily exists because the trick for nice error messages causes conflicting implementations between `T` and `T: Stream` for logical (but annoying) reasons.
/// When Rust supports `T: !Stream` maybe this can be removed.
pub trait IntoSubscriptionResponse<M, TErr> {
    type Stream: Stream<Item = Result<Self::Ok, TErr>>;
    type Ok;
}

const _: () = {
    pub enum Marker {}
    impl<T, TErr, S: Stream<Item = Result<T, TErr>>> IntoSubscriptionResponse<Marker, TErr> for S {
        type Stream = S;
        type Ok = T;
    }
};

const _: () = {
    pub enum Marker {}
    impl<T, S: Stream<Item = Result<T, TErr>>, TErr> IntoSubscriptionResponse<Marker, TErr>
        for Result<S, TErr>
    {
        type Stream = S;
        type Ok = T;
    }
};

const _: () = {
    pub enum Marker {}
    impl<T, TErr, S: Stream<Item = Result<T, TErr>>, F: Future<Output = S>>
        IntoSubscriptionResponse<Marker, TErr> for F
    {
        type Stream = S;
        type Ok = T;
    }
};

// const _: () = {
//     pub enum Marker {}
//     impl<T, S: Stream<Item = Result<T, TErr>>, TErr, F: Future<Output = Result<S, TErr>>>
//         IntoSubscriptionResponse<Marker, TErr> for F
//     {
//         type Stream = Once<F>;
//         type Ok = T;
//     }
// };
