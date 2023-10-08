use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Sink, Stream};

pin_project_lite::pin_project! {
    pub struct SinkAndStream<TSink, TStream> {
        #[pin]
        sink: TSink,
        #[pin]
        stream: TStream,
    }
}

impl<TSink, TStream> SinkAndStream<TSink, TStream> {
    pub fn new(sink: TSink, stream: TStream) -> Self {
        Self { sink, stream }
    }
}

impl<TSink, TStream, TSinkItem> Sink<TSinkItem> for SinkAndStream<TSink, TStream>
where
    TSink: Sink<TSinkItem>,
{
    type Error = TSink::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.sink.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: TSinkItem) -> Result<(), Self::Error> {
        let this = self.project();
        this.sink.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.sink.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.sink.poll_close(cx)
    }
}

impl<TSink, TStream> Stream for SinkAndStream<TSink, TStream>
where
    TStream: Stream,
{
    type Item = TStream::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream.poll_next(cx)
    }
}
