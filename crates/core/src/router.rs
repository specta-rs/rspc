use crate::{internal, Executor};

// TODO: Rename
pub trait IntoRouter: internal::SealedRouter {
    type Ctx;

    fn build(self) -> Executor;
}
