//! rspc-binario: Binario support for rspc
//!
//! TODO:
//!  - Support for streaming the result. Right now we encode into a buffer.
//!  - `BinarioDeserializeError` should end up as a `ProcedureError::Deserialize` not `ProcedureError::Resolver`
//!  - Binario needs impl for `()` for procedures with no input.
//!  - Client integration
//!  - Cleanup HTTP endpoint on `example-binario`. Maybe don't use HTTP cause Axum's model doesn't mesh with Binario?
//!  - Maybe actix-web example to show portability. Might be interesting with the fact that Binario depends on `tokio::AsyncRead`.
//!
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

use std::{error, fmt, marker::PhantomData, pin::Pin};

use binario::{encode, Decode, Encode};
use futures_util::{stream, Stream, StreamExt};
use rspc::{
    middleware::Middleware, DynInput, ProcedureError, ProcedureStream, ResolverInput,
    ResolverOutput,
};
use specta::{datatype::DataType, Generics, Type, TypeCollection};
use tokio::io::AsyncRead;

enum Repr {
    Bytes(Vec<u8>),
    Stream(Pin<Box<dyn AsyncRead + Send>>),
}

pub struct BinarioInput(Repr);

impl BinarioInput {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(Repr::Bytes(bytes))
    }

    pub fn from_stream(stream: impl AsyncRead + Send + 'static) -> Self {
        Self(Repr::Stream(Box::pin(stream)))
    }
}

pub struct TypedBinarioInput<T>(pub BinarioInput, pub PhantomData<T>);

impl<T: Decode + Type + Send + 'static> ResolverInput for TypedBinarioInput<T> {
    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn from_input(input: DynInput) -> Result<Self, ProcedureError> {
        Ok(Self(input.value()?, PhantomData))
    }
}

// TODO: This should probs be a stream not a buffer.
// Binario currently only supports `impl AsyncRead` not `impl Stream`
pub struct BinarioOutput(pub Vec<u8>);
pub struct TypedBinarioOutput<T, M>(pub T, pub PhantomData<fn() -> M>);

pub(crate) mod sealed {
    use super::*;

    pub trait ValidBinarioOutput<M>: Send + 'static {
        type T: Encode + Send + Sync + 'static;
        fn data_type(types: &mut TypeCollection) -> DataType;
        fn into_stream(
            self,
        ) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static;
    }
    pub enum ValueMarker {}
    impl<T: Encode + Type + Send + Sync + 'static> ValidBinarioOutput<ValueMarker> for T {
        type T = T;
        fn data_type(types: &mut TypeCollection) -> DataType {
            T::inline(types, Generics::Definition)
        }
        fn into_stream(
            self,
        ) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
            stream::once(async move { Ok(self) })
        }
    }
    pub enum StreamMarker {}
    impl<S: Stream + Send + Sync + 'static> ValidBinarioOutput<StreamMarker> for rspc::Stream<S>
    where
        S::Item: Encode + Type + Send + Sync + 'static,
    {
        type T = S::Item;

        fn data_type(types: &mut TypeCollection) -> DataType {
            S::Item::inline(types, Generics::Definition)
        }
        fn into_stream(
            self,
        ) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
            self.0.map(|v| Ok(v))
        }
    }
}

impl<TError, M: 'static, T: sealed::ValidBinarioOutput<M>> ResolverOutput<TError>
    for TypedBinarioOutput<T, M>
{
    type T = BinarioOutput;

    fn data_type(types: &mut TypeCollection) -> DataType {
        T::data_type(types)
    }

    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
        // TODO: Encoding into a buffer is not how Binario is intended to work but it's how rspc needs it.
        self.0.into_stream().then(|v| async move {
            let mut buf = Vec::new();
            encode(&v?, &mut buf).await.unwrap(); // TODO: Error handling
            Ok(BinarioOutput(buf))
        })
    }

    fn into_procedure_stream(
        stream: impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream_value(stream)
    }
}

pub fn binario<TError, TCtx, TInput, TResult, M>() -> Middleware<
    TError,
    TCtx,
    TypedBinarioInput<TInput>,
    TypedBinarioOutput<TResult, M>,
    TCtx,
    TInput,
    TResult,
>
where
    TError: From<DeserializeError> + Send + 'static,
    TCtx: Send + 'static,
    TInput: Decode + Send + 'static,
    TResult: sealed::ValidBinarioOutput<M>,
{
    Middleware::new(
        move |ctx: TCtx, input: TypedBinarioInput<TInput>, next| async move {
            let input = match input.0 .0 {
                Repr::Bytes(bytes) => binario::decode::<TInput, _>(bytes.as_slice()).await,
                Repr::Stream(stream) => binario::decode::<TInput, _>(stream).await,
            }
            .map_err(DeserializeError)?;

            next.exec(ctx, input)
                .await
                .map(|v| TypedBinarioOutput(v, PhantomData))
        },
    )
}

pub struct DeserializeError(pub std::io::Error);

impl fmt::Debug for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for DeserializeError {}
