//! rspc-binario: Binario support for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

use std::pin::Pin;

use binario::{encode, Decode, Encode};
use futures::{executor::block_on, Stream};
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

pub struct Binario<T>(pub T);

impl<T: Decode + Type + Send + 'static> ResolverInput for Binario<T> {
    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn from_input(input: DynInput) -> Result<Self, ProcedureError> {
        let stream: BinarioInput = input.value()?;

        // TODO: `block_on` bad
        match stream.0 {
            Repr::Bytes(bytes) => block_on(binario::decode::<T, _>(bytes.as_slice())),
            Repr::Stream(stream) => block_on(binario::decode::<T, _>(stream)),
        }
        .map_err(|err| panic!("{err:?}")) // TODO: Error handling
        .map(Self)
    }
}

// TODO: Streaming instead of this
pub struct BinarioOutput(pub Vec<u8>);

impl<TError, T: Encode + Type + Send + Sync + 'static> ResolverOutput<TError> for Binario<T> {
    type T = BinarioOutput;

    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
        futures::stream::once(async move {
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

pub fn binario<TError, TCtx, TInput, TResult>(
) -> Middleware<TError, TCtx, Binario<TInput>, Binario<TResult>, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Send + 'static,
    TInput: Decode + Send + 'static,
    TResult: Encode + Send + Sync + 'static,
{
    Middleware::new(move |ctx: TCtx, input: Binario<TInput>, next| async move {
        next.exec(ctx, input.0).await.map(Binario)
    })
}
