use std::future::Future;

/// TODO
pub trait AsyncRuntime: 'static {
    /// TODO
    type TaskHandle: Send + 'static;

    /// TODO
    fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) -> Self::TaskHandle;

    /// TODO
    fn cancel_task(task: Self::TaskHandle);
}

#[cfg(feature = "tokio")]
mod tokio_runtime {
    use super::*;

    /// TODO
    pub struct TokioRuntime {}

    impl AsyncRuntime for TokioRuntime {
        type TaskHandle = tokio::task::JoinHandle<()>;

        fn spawn<F: Future<Output = ()> + Send + 'static>(f: F) -> Self::TaskHandle {
            tokio::spawn(f)
        }

        fn cancel_task(task: Self::TaskHandle) {
            task.abort()
        }
    }
}
#[cfg(feature = "tokio")]
pub use tokio_runtime::*;
