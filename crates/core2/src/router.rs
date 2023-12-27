use crate::Executor;

// TODO: Seal this
pub trait Router {
    type Ctx;

    fn build(self) -> Executor;
}
