//! This file contains the magic behind `ProcedureResult`

use std::{any::Any, future::Future, pin::Pin, task::Poll};

// Rust doesn't allow `+` with `dyn` for non-auto traits.
pub(super) trait ErasedSerdeSerializePlusAny:
    erased_serde::Serialize + Any + 'static
{
}
impl<T> ErasedSerdeSerializePlusAny for T where T: erased_serde::Serialize + Any + 'static {}

pub(super) trait AnyErasedFut {
    fn poll(self: Pin<&mut Self>) -> Poll<()>;

    fn take_any(self: Pin<&mut Self>) -> &mut dyn Any;

    fn take_serde(self: Pin<&mut Self>) -> Option<&mut dyn ErasedSerdeSerializePlusAny>;
}

/// TODO: Basically lets us remove lifetime because the value is held with the future and heap allocated together so no extra allocates but no need for a lifetime.
pub(super) enum ErasedFut<F, R> {
    Execute(F),
    Result(Option<R>),
}

impl<F, R> AnyErasedFut for ErasedFut<F, R>
where
    F: Future<Output = R> + 'static,
{
    fn poll(self: Pin<&mut Self>) -> Poll<()> {
        todo!();
    }

    fn take_any(self: Pin<&mut Self>) -> &mut dyn Any {
        todo!()
    }

    fn take_serde(self: Pin<&mut Self>) -> Option<&mut dyn ErasedSerdeSerializePlusAny> {
        todo!()
    }
}
