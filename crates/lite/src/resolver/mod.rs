use std::{future::Ready, marker::PhantomData};

use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

use crate::Executable;

mod executable;

// TODO: Goal:
// - compile time test for support types
// - make good errors for when arg or result are wrong

/// Represents the function defined by an user to handle a specific procedure.
///
/// TODO: Show usage example
///
/// TODO: Potentially make this completely sealed
pub trait ResolverFn<TLCtx, Err, TMarker> {
    type Executable: Executable<TLCtx>;

    /// Convert the resolver function into an [Executable].
    fn executable(self) -> Self::Executable;

    /// Create a type-erased [Executable].
    /// This will likely be slower at runtime but better for compile time!
    fn boxed(self) -> Box<dyn Executable<TLCtx>>
    where
        Self: Sized,
    {
        Box::new(self.executable())
    }
}

// These types intentionally have spelled out generics as they *will* show up in error messages.

// Result<_, _>
const _: () = {
    struct M<A, R>(PhantomData<(A, R)>);
    impl<LayerContext, Argument, Ok, Err, Function>
        ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
    where
        Function: Fn(LayerContext, Argument) -> Result<Ok, Err>,
        Argument: DeserializeOwned + Type + 'static,
        Ok: Serialize + Type + 'static,
    {
        type Executable = executable::Resolver<LayerContext, Argument, Ready<Result<Ok, Err>>>;

        fn executable(self) -> Self::Executable {
            executable::Resolver {}
        }
    }
};

// // impl Future<Output = Result<_, _>>
// const _: () = {
//     struct M<A, R>(PhantomData<(A, R)>);
//     impl<LayerContext, Argument, Ok, Err, Function, Future>
//         ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
//     where
//         Function: Fn(LayerContext, Argument) -> Future,
//         Future: std::future::Future<Output = Result<Ok, Err>>,
//         Argument: DeserializeOwned + Type + 'static,
//         Ok: Serialize + Type + 'static,
//     {
//         type Executable = executable::Resolver;

//         fn executable(self) -> Self::Executable {
//             executable::Resolver {}
//         }
//     }
// };

// // impl Stream<Output = Result<_, _>>
// const _: () = {
//     struct M<A, R>(PhantomData<(A, R)>);
//     impl<LayerContext, Argument, Ok, Err, Function, Future>
//         ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
//     where
//         Function: Fn(LayerContext, Argument) -> Future,
//         Future: futures_core::Stream<Item = Result<Ok, Err>>,
//         Argument: DeserializeOwned + Type + 'static,
//         Ok: Serialize + Type + 'static,
//         // Err, // TODO: Constrain error type
//     {
//         type Executable = executable::Resolver;

//         fn executable(self) -> Self::Executable {
//             executable::Resolver {}
//         }
//     }
// };

// // Result<impl Stream<Output = _>, _>
// const _: () = {
//     struct M<A, R>(PhantomData<(A, R)>);
//     impl<LayerContext, Argument, Ok, Err, Function, Stream>
//         ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
//     where
//         Function: Fn(LayerContext, Argument) -> Result<Stream, Err>,
//         Stream: futures_core::Stream,
//         Argument: DeserializeOwned + Type + 'static,
//         Stream::Item: Serialize + Type + 'static,
//     {
//         type Executable = executable::Resolver;

//         fn executable(self) -> Self::Executable {
//             executable::Resolver {}
//         }
//     }
// };

// impl Future<Output = impl Stream<Output = _>, _>
// const _: () = {
//     struct M<A, R>(PhantomData<(A, R)>);
//     impl<LayerContext, Argument, Ok, Err, Function, Stream>
//         ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
//     where
//         Function: Fn(LayerContext, Argument) -> Result<Stream, Err>,
//         Stream: futures_core::Stream,
//         Argument: DeserializeOwned + Type + 'static,
//         Stream::Item: Serialize + Type + 'static,
//     {
//         type Executable = executable::Resolver;

//         fn executable(self) -> Self::Executable {
//             executable::Resolver {}
//         }
//     }
// };

// impl Future<Output = Result<impl Stream<Output = _>, _>>
// const _: () = {
//     struct M<A, R>(PhantomData<(A, R)>);
//     impl<LayerContext, Argument, Ok, Err, Function, Stream>
//         ResolverFn<LayerContext, Err, M<Argument, Result<Ok, Err>>> for Function
//     where
//         Function: Fn(LayerContext, Argument) -> Result<Stream, Err>,
//         Stream: futures_core::Stream,
//         Argument: DeserializeOwned + Type + 'static,
//         Stream::Item: Serialize + Type + 'static,
//     {
//         type Executable = executable::Resolver;

//         fn executable(self) -> Self::Executable {
//             executable::Resolver {}
//         }
//     }
// };

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_resolver_fn<M, R: ResolverFn<(), (), M>>(r: R) -> R {
        r
    }

    #[allow(unused)]
    fn assert_valid_resolver_fns() {
        asset_resolver_fn(|_, _: ()| Ok("Hello"));
        // asset_resolver_fn(|_, _: ()| "Hello").boxed();

        // TODO
    }
}
