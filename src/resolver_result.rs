use std::{future::Future, marker::PhantomData, pin::Pin};

use serde::Serialize;
use serde_json::Value;

pub enum ResolverResult {
    Future(Pin<Box<dyn Future<Output = Value>>>),
    Value(Value),
}

pub trait IntoResolverResult<TMarker> {
    fn into_resolver_result(self) -> ResolverResult;
}

pub struct ValueMarker(PhantomData<()>);
impl<T> IntoResolverResult<ValueMarker> for T
where
    T: Serialize,
{
    fn into_resolver_result(self) -> ResolverResult {
        ResolverResult::Value(serde_json::to_value(self).unwrap())
    }
}

pub struct FutureMarker(PhantomData<()>);
impl<TFut, T> IntoResolverResult<FutureMarker> for TFut
where
    TFut: Future<Output = T> + 'static,
    T: Serialize,
{
    fn into_resolver_result(self) -> ResolverResult {
        ResolverResult::Future(Box::pin(async move {
            serde_json::to_value(self.await).unwrap()
        }))
    }
}
