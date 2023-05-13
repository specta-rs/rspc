mod private {
    use crate::internal::exec::ExecutorBase;

    pub struct Executor<E: ExecutorBase> {
        executor: E,
    }

    impl<E: ExecutorBase> Executor<E> {
        pub fn new(executor: E) -> Self {
            Self { executor }
        }

        /// TODO
        pub fn execute(&self) {
            // TODO
        }
    }
}

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::Executor;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::Executor;
