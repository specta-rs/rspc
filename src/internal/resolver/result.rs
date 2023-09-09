use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::internal::Body;
use futures::Stream;

use serde_json::Value;

use crate::ExecError;

mod private {
    use pin_project_lite::pin_project;

    use super::*;

    pin_project! {
        // TODO: Try and remove this?
        pub struct StreamAdapter<S> {
            #[pin] // TODO: Remove `pub(crate)`
            pub(crate) stream: S,
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
}

// TODO: Temporary
pub(crate) use private::StreamAdapter;
