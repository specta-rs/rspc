use std::{future::Future, time::Instant};

/// TODO
pub trait AsyncRuntime: 'static {
    /// TODO
    type TaskHandle: Send + 'static;

    /// TODO
    type SleepUtilFut: Future<Output = ()> + Send + 'static;

    /// TODO
    fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) -> Self::TaskHandle;

    /// TODO
    fn cancel_task(task: Self::TaskHandle);

    /// TODO
    fn sleep_util(till: Instant) -> Self::SleepUtilFut;
}

#[cfg(feature = "tokio")]
mod tokio_runtime {
    use super::*;

    /// TODO
    pub struct TokioRuntime {}

    impl AsyncRuntime for TokioRuntime {
        type TaskHandle = tokio::task::JoinHandle<()>;
        type SleepUtilFut = tokio::time::Sleep;

        fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) -> Self::TaskHandle {
            tokio::spawn(f)
        }

        fn cancel_task(task: Self::TaskHandle) {
            task.abort()
        }

        fn sleep_util(till: Instant) -> Self::SleepUtilFut {
            tokio::time::sleep_until(till.into())
        }
    }
}

#[cfg(feature = "tokio")]
pub use tokio_runtime::*;
