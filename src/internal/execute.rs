use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    internal::{OperationKey, OperationKind, RequestInner, ResponseInner},
    ExecError,
};

use super::{Procedure, Request, Response};

pub trait RequestRouter {
    type Ctx: 'static;

    fn queries(&self) -> &BTreeMap<String, Procedure<Self::Ctx>>;

    fn mutations(&self) -> &BTreeMap<String, Procedure<Self::Ctx>>;

    fn subscriptions(&self) -> &BTreeMap<String, Procedure<Self::Ctx>>;
}

impl Request {
    /// TODO: Docs
    pub async fn execute<R: RequestRouter>(
        self,
        r: &R,
        ctx: R::Ctx,
        // TODO: Don't return result -> Map to response
    ) -> Result<Value, ExecError> {
        if !self.jsonrpc.is_none() && self.jsonrpc.as_deref() != Some("2.0") {
            return Err(ExecError::InvalidJsonRpcVersion);
        }

        let (path, input, procedures) = match self.inner {
            RequestInner::Query { path, input } => (path, input, r.queries()),
            RequestInner::Mutation { path, input } => (path, input, r.mutations()),
            RequestInner::Subscription { path, input } => {
                Err(ExecError::UnsupportedMethod("subscription"))?
            }
            RequestInner::StopSubscription => {
                Err(ExecError::UnsupportedMethod("stopSubscription"))?
            }
        };

        procedures
            .get(&path)
            .ok_or(ExecError::OperationNotFound(path))?
            .exec
            .call(
                ctx,
                input.unwrap_or(Value::Null),
                (OperationKind(), OperationKey()),
            )?
            .into_value()
            .await
    }

    /// TODO: Docs
    pub async fn execute_response<R: RequestRouter>(
        self,
        r: &R,
        ctx: R::Ctx,
        // TODO: Don't return result -> Map to response
    ) -> Response {
        Response {
            jsonrpc: "2.0",
            id: self.id.clone(),
            inner: match self.execute(r, ctx).await {
                Ok(result) => ResponseInner::Ok { result },
                Err(err) => ResponseInner::Err { error: err.into() },
            },
        }
    }

    /// TODO: Docs
    pub fn execute_stream<R: RequestRouter>(self, r: &R, ctx: R::Ctx) {
        match self.inner {
            RequestInner::Query { path, input } => todo!(),
            RequestInner::Mutation { path, input } => todo!(),
            RequestInner::Subscription { path, input } => todo!(),
            RequestInner::StopSubscription => todo!(),
        }
    }
}
