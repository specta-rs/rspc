use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    task::{ready, Context, Poll},
};

use futures::{
    stream::{once, Once},
    Stream,
};
use serde::Serialize;
use serde_json::Value;
use specta::Type;

use crate::{Error, ExecError};

#[doc(hidden)]
pub trait RequestLayer<TMarker>: private::SealedRequestLayer<TMarker> {}

mod private {
    use std::convert::Infallible;

    use pin_project_lite::pin_project;

    use crate::{internal::Body, Blob};

    use super::*;

    pin_project! {
        // TODO: Try and remove
        pub struct StreamAdapter<S> {
            #[pin]
            stream: S,
        }
    }

    impl<S: Stream<Item = Result<Value, ExecError>> + Send + 'static> Body for StreamAdapter<S> {
        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Value, ExecError>>> {
            self.project().stream.poll_next(cx)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    // Markers
    #[doc(hidden)]
    pub enum StreamMarkerType {}
    #[doc(hidden)]
    pub enum FutureMarkerType {}

    pub trait SealedRequestLayer<TMarker> {
        type Result: Type;
        type Body: Body + Send + 'static;
        type TypeMarker;

        fn exec(self) -> Self::Body;
    }

    impl<TMarker, T: SealedRequestLayer<TMarker>> RequestLayer<TMarker> for T {}

    // For queries and mutations

    #[doc(hidden)]
    pub enum SerializeMarker {}
    impl<T> SealedRequestLayer<SerializeMarker> for T
    where
        T: Serialize + Type,
    {
        type Result = T;
        type Body = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: once(ready(
                    serde_json::to_value(self).map_err(ExecError::SerializingResultErr),
                )),
            }
        }
    }

    // TODO: Allow `Blob<T>` with `futures::AsyncRead`/`futures:AsyncBufRead` traits

    #[doc(hidden)]
    pub enum BlobAsyncBufReadMarker {}
    #[cfg(feature = "tokio")]
    impl<S> SealedRequestLayer<BlobAsyncBufReadMarker> for Blob<S>
    where
        S: tokio::io::AsyncBufRead + Send + 'static,
    {
        type Result = ();
        type Body = BlobStream<S>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            BlobStream { stream: self.0 }
        }
    }

    #[doc(hidden)]
    pub enum ResultMarker {}
    impl<T> SealedRequestLayer<ResultMarker> for Result<T, Error>
    where
        T: Serialize + Type,
    {
        type Result = T;
        type Body = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: once(ready(self.map_err(ExecError::ErrResolverError).and_then(
                    |v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
                ))),
            }
        }
    }

    #[doc(hidden)]
    pub enum FutureSerializeMarker {}
    impl<F> SealedRequestLayer<FutureSerializeMarker> for F
    where
        F: Future + Send + 'static,
        F::Output: Serialize + Type + Send + 'static,
    {
        type Result = F::Output;
        type Body = StreamAdapter<Once<FutureSerializeFuture<F>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: once(FutureSerializeFuture { fut: self }),
            }
        }
    }

    #[doc(hidden)]
    pub struct FutureBlobAsyncBufReadMarker<S>(
        PhantomData<S>,
        // Prevents this type from being instantiated
        Infallible,
    );
    #[cfg(feature = "tokio")]
    impl<TFut, S> SealedRequestLayer<FutureBlobAsyncBufReadMarker<S>> for TFut
    where
        TFut: Future<Output = Blob<S>> + Send + 'static,
        S: tokio::io::AsyncBufRead + Send + 'static,
    {
        type Result = ();
        type Body = FutureBlobStream<TFut, S>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            FutureBlobStream {
                fut: self,
                map: |v| v.0,
                phantom: PhantomData,
            }
        }
    }

    pin_project! {
        #[project = FutureSerializeFutureProj]
        pub struct FutureSerializeFuture<TFut> {
            #[pin]
            fut: TFut,
        }
    }

    impl<TFut> Future for FutureSerializeFuture<TFut>
    where
        TFut: Future + Send + 'static,
        TFut::Output: Serialize + Type + Send + 'static,
    {
        type Output = Result<Value, ExecError>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project()
                .fut
                .poll(cx)
                .map(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr))
        }
    }

    #[doc(hidden)]
    pub enum FutureResultMarker {}
    impl<TFut, T> SealedRequestLayer<FutureResultMarker> for TFut
    where
        TFut: Future<Output = Result<T, Error>> + Send + 'static,
        T: Serialize + Type + Send + 'static,
    {
        type Result = T;
        type Body = StreamAdapter<Once<FutureSerializeResultFuture<TFut, T>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: once(FutureSerializeResultFuture {
                    fut: self,
                    phantom: PhantomData,
                }),
            }
        }
    }

    pin_project! {
        #[project = FutureSerializeResultFutureProj]
        pub struct FutureSerializeResultFuture<TFut, T> {
            #[pin]
            fut: TFut,
            phantom: PhantomData<T>
        }
    }

    impl<TFut, T> Future for FutureSerializeResultFuture<TFut, T>
    where
        TFut: Future<Output = Result<T, Error>> + Send + 'static,
        T: Serialize + Type + Send + 'static,
    {
        type Output = Result<Value, ExecError>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project().fut.poll(cx).map(|v| {
                v.map_err(ExecError::ErrResolverError)
                    .and_then(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr))
            })
        }
    }

    // For subscriptions

    #[doc(hidden)]
    pub enum StreamMarker {}
    impl<S> SealedRequestLayer<StreamMarker> for S
    where
        S: Stream + Send + Sync + 'static,
        S::Item: Serialize + Type,
    {
        type Result = S::Item;
        type Body = StreamAdapter<MapStream<S>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: MapStream::Stream {
                    stream: self,
                    mapper: |v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
                },
            }
        }
    }

    #[doc(hidden)]
    pub enum ResultStreamMarker {}
    impl<S> SealedRequestLayer<ResultStreamMarker> for Result<S, Error>
    where
        S: Stream + Send + Sync + 'static,
        S::Item: Serialize + Type,
    {
        type Result = S::Item;
        type Body = StreamAdapter<MapStream<S>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: match self {
                    Ok(stream) => MapStream::Stream {
                        stream,
                        mapper: |v| {
                            serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
                        },
                    },
                    Err(err) => MapStream::Error {
                        err: Some(ExecError::ErrResolverError(err)),
                    },
                },
            }
        }
    }

    #[doc(hidden)]
    pub enum StreamResultMarker {}
    impl<TStream, T> SealedRequestLayer<StreamResultMarker> for TStream
    where
        TStream: Stream<Item = Result<T, Error>> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Body = StreamAdapter<MapStream<TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: MapStream::Stream {
                    stream: self,
                    mapper: |v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
                },
            }
        }
    }

    #[doc(hidden)]
    pub enum FutureStreamMarker {}
    impl<TFut, S> SealedRequestLayer<FutureStreamMarker> for TFut
    where
        TFut: Future<Output = S> + Send + 'static,
        S: Stream + Send + Sync + 'static,
        S::Item: Serialize + Type,
    {
        type Result = S::Item;
        type Body = StreamAdapter<FutureMapStream<TFut, S>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: FutureMapStream::First {
                    fut: self,
                    fut_mapper: Ok,
                    stream_mapper: |v| {
                        serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
                    },
                },
            }
        }
    }

    #[doc(hidden)]
    pub enum FutureResultStreamMarker {}
    impl<TFut, S> SealedRequestLayer<FutureResultStreamMarker> for TFut
    where
        TFut: Future<Output = Result<S, Error>> + Send + 'static,
        S: Stream + Send + Sync + 'static,
        S::Item: Serialize + Type,
    {
        type Result = S::Item;
        type Body = StreamAdapter<FutureMapStream<TFut, S>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: FutureMapStream::First {
                    fut: self,
                    fut_mapper: |s| s.map_err(ExecError::ErrResolverError),
                    stream_mapper: |v| {
                        serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
                    },
                },
            }
        }
    }

    #[doc(hidden)]
    pub enum FutureStreamResultMarker {}
    impl<TFut, TStream, T> SealedRequestLayer<FutureStreamResultMarker> for TFut
    where
        TFut: Future<Output = TStream> + Send + 'static,
        TStream: Stream<Item = Result<T, Error>> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Body = StreamAdapter<FutureMapStream<TFut, TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Body {
            StreamAdapter {
                stream: FutureMapStream::First {
                    fut: self,
                    fut_mapper: Ok,
                    stream_mapper: |v| {
                        serde_json::to_value(v).map_err(ExecError::SerializingResultErr)
                    },
                },
            }
        }
    }

    pin_project! {
        #[project = MapStreamEnumProj]
        pub enum MapStream<S: Stream> {
            Stream {
                #[pin]
                stream: S,
                mapper: fn(S::Item) -> Result<Value, ExecError>,
            },
            Error {
                // Optional to allow value to be removed on first poll
                err: Option<ExecError>,
            },
        }
    }

    impl<S: Stream> Stream for MapStream<S> {
        type Item = Result<Value, ExecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let return_value = match self.as_mut().project() {
                MapStreamEnumProj::Error { err } => Poll::Ready(err.take().map(Err)),
                MapStreamEnumProj::Stream { stream, mapper } => {
                    stream.poll_next(cx).map(|result| result.map(mapper))
                }
            };

            return_value
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                Self::Stream { stream, .. } => stream.size_hint(),
                _ => (0, Some(0)),
            }
        }
    }

    pin_project! {
        // TODO: Document phases
        #[project = FutureMapStreamProj]
        pub enum FutureMapStream<F: Future, S: Stream> {
            First {
                #[pin]
                fut: F,
                fut_mapper: fn(F::Output) -> Result<S, ExecError>,
                stream_mapper: fn(S::Item) -> Result<Value, ExecError>,
            },
            Second {
                #[pin]
                stream: S,
                stream_mapper: fn(S::Item) -> Result<Value, ExecError>,
            },
        }
    }

    impl<F: Future, S: Stream> Stream for FutureMapStream<F, S> {
        type Item = Result<Value, ExecError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            loop {
                let new_value = match self.as_mut().project() {
                    FutureMapStreamProj::First {
                        fut,
                        fut_mapper,
                        stream_mapper,
                    } => {
                        let result = ready!(fut.poll(cx));

                        match (fut_mapper)(result) {
                            Ok(stream) => Self::Second {
                                stream,
                                stream_mapper: *stream_mapper,
                            },
                            Err(err) => return Poll::Ready(Some(Err(err))),
                        }
                    }
                    FutureMapStreamProj::Second {
                        stream,
                        stream_mapper,
                    } => return stream.poll_next(cx).map(|result| result.map(stream_mapper)),
                };

                self.as_mut().set(new_value);
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match self {
                Self::First { .. } => (0, None),
                Self::Second { stream, .. } => stream.size_hint(),
            }
        }
    }

    pin_project! {
        #[project = FutureBlobStreamProj]
        pub struct FutureBlobStream<F, S>
        where
            F: Future
        {
            #[pin]
            fut: F,
            map: fn(F::Output) -> S,
            phantom: PhantomData<S>
        }
    }

    impl<S: tokio::io::AsyncBufRead, F: Future> Body for FutureBlobStream<F, S> {
        fn poll_next(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Value, ExecError>>> {
            todo!("blob unimplemented");
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    pin_project! {
        #[project = BlobStreamProj]
        pub struct BlobStream<S> {
            #[pin]
            stream: S,
            // buf: Vec<u8>,
        }
    }

    impl<S: tokio::io::AsyncBufRead> Body for BlobStream<S> {
        fn poll_next(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Value, ExecError>>> {
            todo!("blob unimplemented");
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }
}

pub(crate) use private::{FutureMarkerType, StreamMarkerType};
