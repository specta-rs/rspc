use crate::{internal, Executor};

// TODO: Rename
pub trait Router: internal::SealedRouter {
    type Ctx;

    fn build(self) -> Executor;
}
