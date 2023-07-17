use std::{future::Future, time::Instant};

/// Define an async runtime.
pub trait AsyncRuntime: Sync + Send + 'static {
    /// A handle to a task that was spawned using `Self::spawn`.
    /// This must be able to cancel the task.
    type TaskHandle: Send + 'static;

    /// A future that can be used to sleep until a given instant.
    type SleepUtilFut: Future<Output = ()> + Send + 'static;

    /// spawn a future onto the async runtime and return a handle to it.
    fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) -> Self::TaskHandle;

    /// cancel a task given its handle. This should end execution and `Drop` it.
    fn cancel_task(task: Self::TaskHandle);

    /// returns a future that waits until an instant to resolve.
    fn sleep_util(till: Instant) -> Self::SleepUtilFut;
}

#[cfg(feature = "tokio")]
mod tokio_runtime {
    use super::*;

    /// Support for the [Tokio](https://tokio.rs/) async runtime.
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
