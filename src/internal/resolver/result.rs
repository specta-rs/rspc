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

    use bytes::Bytes;
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
        type Stream: Body + Send + 'static;
        type TypeMarker;

        fn exec(self) -> Self::Stream;
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
        type Stream = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
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
        S: tokio::io::AsyncBufRead,
    {
        type Result = ();
        type Stream = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
            todo!();
        }
    }

    #[doc(hidden)]
    pub enum ResultMarker {}
    impl<T> SealedRequestLayer<ResultMarker> for Result<T, Error>
    where
        T: Serialize + Type,
    {
        type Result = T;
        type Stream = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
            StreamAdapter {
                stream: once(ready(self.map_err(ExecError::ErrResolverError).and_then(
                    |v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
                ))),
            }
        }
    }

    #[doc(hidden)]
    pub enum FutureSerializeMarker {}
    impl<TFut, T> SealedRequestLayer<FutureSerializeMarker> for TFut
    where
        TFut: Future<Output = T> + Send + 'static,
        T: Serialize + Type + Send + 'static,
    {
        type Result = T;
        type Stream = StreamAdapter<Once<FutureSerializeFuture<TFut, T>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
            StreamAdapter {
                stream: once(FutureSerializeFuture {
                    fut: self,
                    phantom: PhantomData,
                }),
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
        S: tokio::io::AsyncBufRead,
    {
        type Result = ();
        type Stream = StreamAdapter<Once<Ready<Result<Value, ExecError>>>>; // StreamAdapter<Once<FutureSerializeFuture<TFut, T>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
            todo!();
        }
    }

    pin_project! {
        #[project = FutureSerializeFutureProj]
        pub struct FutureSerializeFuture<TFut, T> {
            #[pin]
            fut: TFut,
            phantom: PhantomData<T>
        }
    }

    impl<TFut, T> Future for FutureSerializeFuture<TFut, T>
    where
        TFut: Future<Output = T> + Send + 'static,
        T: Serialize + Type + Send + 'static,
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
        type Stream = StreamAdapter<Once<FutureSerializeResultFuture<TFut, T>>>;
        type TypeMarker = FutureMarkerType;

        fn exec(self) -> Self::Stream {
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
            // self.project().fut.poll(cx).map(|v| {
            //     v.map_err(ExecError::ErrResolverError)
            //         .and_then(|v| serde_json::to_value(v).map_err(ExecError::SerializingResultErr))
            // })
            todo!();
        }
    }

    // For subscriptions

    #[doc(hidden)]
    pub enum StreamMarker {}
    impl<TStream, T> SealedRequestLayer<StreamMarker> for TStream
    where
        TStream: Stream<Item = T> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Stream = StreamAdapter<MapStream<TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
    impl<TStream, T> SealedRequestLayer<ResultStreamMarker> for Result<TStream, Error>
    where
        TStream: Stream<Item = T> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Stream = StreamAdapter<MapStream<TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
        type Stream = StreamAdapter<MapStream<TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
    impl<TFut, TStream, T> SealedRequestLayer<FutureStreamMarker> for TFut
    where
        TFut: Future<Output = TStream> + Send + 'static,
        TStream: Stream<Item = T> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Stream = StreamAdapter<FutureMapStream<TFut, TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
    impl<TFut, TStream, T> SealedRequestLayer<FutureResultStreamMarker> for TFut
    where
        TFut: Future<Output = Result<TStream, Error>> + Send + 'static,
        TStream: Stream<Item = T> + Send + Sync + 'static,
        T: Serialize + Type,
    {
        type Result = T;
        type Stream = StreamAdapter<FutureMapStream<TFut, TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
        type Stream = StreamAdapter<FutureMapStream<TFut, TStream>>;
        type TypeMarker = StreamMarkerType;

        fn exec(self) -> Self::Stream {
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
}

pub(crate) use private::{FutureMarkerType, StreamMarkerType};
