use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{KeyDefinition, Router};

// TODO: Rename this type
// TODO: Export this and use on frontend?
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageMethod {
    Query,
    Mutation,
    SubscriptionAdd,
    SubscriptionRemove,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Option<String>,
    pub method: MessageMethod,
    pub operation: String,
    pub arg: Option<Value>,
}

impl Request {
    pub async fn handle<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>(
        self,
        ctx: TCtx,
        router: &Arc<Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>>,
    ) -> Response
    where
        TCtx: Send + 'static,
        TMeta: Send + Sync + 'static,
        TQueryKey: KeyDefinition,
        TMutationKey: KeyDefinition,
        TSubscriptionKey: KeyDefinition,
    {
        match self.method {
            MessageMethod::Query => {
                match router
                    .exec_query_unsafe(ctx, self.operation, self.arg.unwrap_or(Value::Null))
                    .await
                {
                    Ok(result) => Response::Response(ResponseResult::Success {
                        id: self.id,
                        result: result,
                    }),
                    Err(_) => Response::Response(ResponseResult::Error),
                }
            }
            MessageMethod::Mutation => {
                match router
                    .exec_mutation_unsafe(ctx, self.operation, self.arg.unwrap_or(Value::Null))
                    .await
                {
                    Ok(result) => Response::Response(ResponseResult::Success {
                        id: self.id,
                        result: result,
                    }),
                    Err(_) => Response::Response(ResponseResult::Error),
                }
            }
            MessageMethod::SubscriptionAdd => {
                // match router
                //     .add_subscription_unsafe(ctx, self.operation, self.arg.unwrap_or(Value::Null))
                //     .await
                // {
                //     Ok(result) => Response::Response(ResponseResult::Success {
                //         id: self.id,
                //         result: result,
                //     }),
                //     Err(_) => Response::Response(ResponseResult::Error),
                // }
                unimplemented!()
            }
            MessageMethod::SubscriptionRemove => {
                unimplemented!();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    Event, // TODO: Make events work
    Response(ResponseResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResponseResult {
    Success { id: Option<String>, result: Value },
    Error, // TODO: Make events work
}
