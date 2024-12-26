//! rspc-binario: Binario support for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

use std::{error, fmt, marker::PhantomData, pin::Pin};

use binario::{encode, Decode, Encode};
use futures_util::{stream, Stream};
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

// TODO: Streaming instead of this
pub struct BinarioOutput(pub Vec<u8>);
pub struct TypedBinarioOutput<T>(pub T);

impl<TError, T: Encode + Type + Send + Sync + 'static> ResolverOutput<TError>
    for TypedBinarioOutput<T>
{
    type T = BinarioOutput;

    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
        stream::once(async move {
            let mut buf = Vec::new();
            encode(&self.0, &mut buf).await.unwrap(); // TODO: Error handling
            Ok(BinarioOutput(buf))
        })
    }

    fn into_procedure_stream(
        stream: impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream_value(stream)
    }
}

pub fn binario<TError, TCtx, TInput, TResult>() -> Middleware<
    TError,
    TCtx,
    TypedBinarioInput<TInput>,
    TypedBinarioOutput<TResult>,
    TCtx,
    TInput,
    TResult,
>
where
    TError: From<DeserializeError> + Send + 'static,
    TCtx: Send + 'static,
    TInput: Decode + Send + 'static,
    TResult: Encode + Send + Sync + 'static,
{
    Middleware::new(
        move |ctx: TCtx, input: TypedBinarioInput<TInput>, next| async move {
            let input = match input.0 .0 {
                Repr::Bytes(bytes) => binario::decode::<TInput, _>(bytes.as_slice()).await,
                Repr::Stream(stream) => binario::decode::<TInput, _>(stream).await,
            }
            .map_err(DeserializeError)?;

            next.exec(ctx, input).await.map(TypedBinarioOutput)
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
