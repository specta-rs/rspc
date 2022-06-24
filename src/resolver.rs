use std::{future::Future, path::PathBuf, pin::Pin};

use serde::Serialize;
use serde_json::Value;
use ts_rs::{Dependency, ExportError, TS};

pub enum ResolverResult {
    Value(Value),
    Future(Pin<Box<dyn Future<Output = ResolverResult>>>),
}

pub trait Resolver<TType> {
    fn resolve(self) -> ResolverResult;

    fn export(export_path: PathBuf) -> Result<(String, Vec<Dependency>), ExportError>;
}

pub struct SerdeType;
impl<TValue: Serialize + TS> Resolver<SerdeType> for TValue {
    fn resolve(self) -> ResolverResult {
        ResolverResult::Value(serde_json::to_value(self).unwrap()) // TODO: handle unwrap
    }

    fn export(export_path: PathBuf) -> Result<(String, Vec<Dependency>), ExportError> {
        // TODO: This is a very suboptiomal solution for https://github.com/Aleph-Alpha/ts-rs/issues/70
        let type_name = match <TValue as TS>::transparent() {
            true => <TValue as TS>::inline(),
            false => <TValue as TS>::name(),
        };

        match <TValue as TS>::export_to(export_path.join(format!("{}.ts", <TValue as TS>::name())))
        {
            Ok(_) | Err(ExportError::CannotBeExported) => {
                Ok((type_name, <TValue as TS>::dependencies()))
            }
            Err(v) => Err(v),
        }
    }
}

pub struct FutureType<TRetType>(TRetType);
impl<TRetType: 'static, TRet: Resolver<TRetType>, TFut: Future<Output = TRet> + 'static>
    Resolver<FutureType<TRetType>> for TFut
{
    fn resolve(self) -> ResolverResult {
        ResolverResult::Future(Box::pin(async move { self.await.resolve() }))
    }

    fn export(export_path: PathBuf) -> Result<(String, Vec<Dependency>), ExportError> {
        TRet::export(export_path)
    }
}
