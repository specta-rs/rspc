use std::{future::Future, pin::Pin};

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

pub enum ResolverResult {
    Value(Value),
    Future(Pin<Box<dyn Future<Output = ResolverResult>>>),
}

pub trait Resolver<TType> {
    fn resolve(self) -> ResolverResult;
}

pub struct SerdeType;
impl<TValue: Serialize + TS> Resolver<SerdeType> for TValue {
    fn resolve(self) -> ResolverResult {
        ResolverResult::Value(serde_json::to_value(self).unwrap()) // TODO: handle unwrap
    }
}

pub struct FutureType<TRetType>(TRetType);
impl<TRetType: 'static, TRet: Resolver<TRetType>, TFut: Future<Output = TRet> + 'static>
    Resolver<FutureType<TRetType>> for TFut
{
    fn resolve(self) -> ResolverResult {
        ResolverResult::Future(Box::pin(async move { self.await.resolve() }))
    }
}
