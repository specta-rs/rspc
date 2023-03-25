use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, StreamExt};
use pin_project::pin_project;
use serde::Serialize;
use specta::Type;

use crate::{internal::ValueOrStream, Error, ExecError};

// TODO: Replace `ValueOrStream` with assoicated type and trait on it to make it work

// For queries and mutations

pub trait RequestLayer<TMarker> {
    type Result: Type;
    type Fut: Future<Output = Result<ValueOrStream, ExecError>> + Send + 'static;

    // TODO: Rename func
    fn into_layer_result(self) -> Self::Fut;
}

pub enum SerializeMarker {}
impl<T> RequestLayer<SerializeMarker> for T
where
    T: Serialize + Type,
{
    type Result = T;
    type Fut = Ready<Result<ValueOrStream, ExecError>>;

    fn into_layer_result(self) -> Self::Fut {
        ready(
            serde_json::to_value(self)
                .map(ValueOrStream::Value)
                .map_err(ExecError::SerializingResultErr),
        )
    }
}

pub enum ResultMarker {}
impl<T> RequestLayer<ResultMarker> for Result<T, Error>
where
    T: Serialize + Type,
{
    type Result = T;
    type Fut = Ready<Result<ValueOrStream, ExecError>>;

    fn into_layer_result(self) -> Self::Fut {
        ready(self.map_err(ExecError::ErrResolverError).and_then(|v| {
            serde_json::to_value(v)
                .map(ValueOrStream::Value)
                .map_err(ExecError::SerializingResultErr)
        }))
    }
}

pub enum FutureSerializeMarker {}
impl<TFut, T> RequestLayer<FutureSerializeMarker> for TFut
where
    TFut: Future<Output = T> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;
    type Fut = FutureResultFuture<TFut, T>;

    fn into_layer_result(self) -> Self::Fut {
        FutureResultFuture(self, PhantomData)
        // Ok(LayerResult::Future(Box::pin(async move {
        //     match self
        //         .await
        //         .into_layer_result()?
        //         .into_value_or_stream()
        //         .await?
        //     {
        //         ValueOrStream::Stream(_) => unreachable!(),
        //         ValueOrStream::Value(v) => Ok(v),
        //     }
        // })))

        // ready(self.map_err(ExecError::ErrResolverError).and_then(|v| {
        //     serde_json::to_value(v)
        //         .map(ValueOrStream::Value)
        //         .map_err(ExecError::SerializingResultErr)
        // }))
    }
}

#[pin_project(project = FutureResultFutProj)]
pub struct FutureResultFuture<TFut, T>(#[pin] TFut, PhantomData<T>);

impl<TFut, T> Future for FutureResultFuture<TFut, T>
where
    TFut: Future<Output = T> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Output = Result<ValueOrStream, ExecError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.0.poll(cx) {
            Poll::Ready(v) => {
                todo!(); // `v.into_layer_result()?` then repeat

                // Poll::Ready(
                //     v.and_then(|v| {
                //         serde_json::to_value(v)
                //             .map(ValueOrStream::Value)
                //             .map_err(ExecError::SerializingResultErr)
                //     }), // serde_json::to_value(v)
                //         //     .map(ValueOrStream::Value)
                //         //     .map_err(ExecError::SerializingResultErr),
                // )
            }
            // Poll::Ready(Err(e)) => Poll::Ready(Err(ExecError::ErrResolverError(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub enum FutureResultMarker {}
impl<TFut, T> RequestLayer<FutureResultMarker> for TFut
where
    TFut: Future<Output = Result<T, Error>> + Send + 'static,
    T: Serialize + Type + Send + 'static,
{
    type Result = T;
    type Fut = Ready<Result<ValueOrStream, ExecError>>; // TODO

    fn into_layer_result(self) -> Self::Fut {
        // Ok(LayerResult::Future(Box::pin(async move {
        //     match self
        //         .await
        //         .into_layer_result()?
        //         .into_value_or_stream()
        //         .await?
        //     {
        //         ValueOrStream::Stream(_) => unreachable!(),
        //         ValueOrStream::Value(v) => Ok(v),
        //     }
        // })))
        todo!();
    }
}

// // For subscriptions

// pub trait StreamRequestLayer<TMarker> {
//     type Result: Type;
//     // TODO: type Stream = TODO;

//     fn into_layer_result(self) -> Result<LayerResult, ExecError>;
// }

// pub enum StreamMarker {}
// impl<TStream, T> StreamRequestLayer<StreamMarker> for TStream
// where
//     TStream: Stream<Item = T> + Send + Sync + 'static,
//     T: Serialize + Type,
// {
//     type Result = T;

//     fn into_layer_result(self) -> Result<LayerResult, ExecError> {
//         Ok(LayerResult::Stream(Box::pin(self.map(|v| {
//             serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
//         }))))
//     }
// }

// pub enum ResultStreamMarker {}
// impl<TStream, T> StreamRequestLayer<ResultStreamMarker> for Result<TStream, Error>
// where
//     TStream: Stream<Item = T> + Send + Sync + 'static,
//     T: Serialize + Type,
// {
//     type Result = T;

//     fn into_layer_result(self) -> Result<LayerResult, ExecError> {
//         Ok(LayerResult::Stream(Box::pin(
//             self.map_err(ExecError::ErrResolverError)?
//                 .map(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr)),
//         )))
//     }
// }

// pub enum FutureStreamMarker {}
// impl<TFut, TStream, T> StreamRequestLayer<FutureStreamMarker> for TFut
// where
//     TFut: Future<Output = TStream> + Send + 'static,
//     TStream: Stream<Item = T> + Send + Sync + 'static,
//     T: Serialize + Type,
// {
//     type Result = T;

//     fn into_layer_result(self) -> Result<LayerResult, ExecError> {
//         Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
//             Ok(ValueOrStream::Stream(Box::pin(self.await.map(|v| {
//                 serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
//             }))))
//         })))
//     }
// }

// pub enum FutureResultStreamMarker {}
// impl<TFut, TStream, T> StreamRequestLayer<FutureResultStreamMarker> for TFut
// where
//     TFut: Future<Output = Result<TStream, Error>> + Send + 'static,
//     TStream: Stream<Item = T> + Send + Sync + 'static,
//     T: Serialize + Type,
// {
//     type Result = T;

//     fn into_layer_result(self) -> Result<LayerResult, ExecError> {
//         Ok(LayerResult::FutureValueOrStream(Box::pin(async move {
//             Ok(ValueOrStream::Stream(Box::pin(
//                 self.await
//                     .map_err(ExecError::ErrResolverError)?
//                     .map(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr)),
//             )))
//         })))
//     }
// }
