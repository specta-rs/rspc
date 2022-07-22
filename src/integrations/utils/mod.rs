use std::sync::Arc;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{spawn, sync::mpsc::UnboundedSender};

use crate::{KeyDefinition, Router};

/// TODO
#[derive(Serialize, Deserialize, Debug)]
pub struct OperationKey(
    pub String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub Option<Value>,
);

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

/// TODO
#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Option<String>,
    pub operation: MessageMethod,
    pub key: OperationKey,
}

impl Request {
    pub async fn handle<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>(
        mut self,
        ctx: TCtx,
        router: &Arc<Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>>,
        event_sender: Option<&UnboundedSender<Response>>,
    ) -> Response
    where
        TCtx: Send + 'static,
        TMeta: Send + Sync + 'static,
        TQueryKey: KeyDefinition,
        TMutationKey: KeyDefinition,
        TSubscriptionKey: KeyDefinition,
    {
        if let Some(Value::Object(obj)) = &self.key.1 {
            if obj.len() == 0 {
                self.key.1 = Some(Value::Null);
            }
        }

        match self.operation {
            MessageMethod::Query => {
                match router
                    .exec_query_unsafe(
                        ctx,
                        self.key.0,
                        self.key.1.unwrap_or(Value::Null),
                        self.key.2.unwrap_or(Value::Null),
                    )
                    .await
                {
                    Ok(result) => Response::Response(ResponseResult::Success {
                        id: self.id,
                        result: result,
                    }),
                    Err(err) => {
                        println!("Error: {}", err); // TODO: Proper error handling
                        Response::Response(ResponseResult::Error)
                    }
                }
            }
            MessageMethod::Mutation => {
                match router
                    .exec_mutation_unsafe(
                        ctx,
                        self.key.0,
                        self.key.1.unwrap_or(Value::Null),
                        self.key.2.unwrap_or(Value::Null),
                    )
                    .await
                {
                    Ok(result) => Response::Response(ResponseResult::Success {
                        id: self.id,
                        result: result,
                    }),
                    Err(err) => {
                        println!("Error: {}", err); // TODO: Proper error handling
                        Response::Response(ResponseResult::Error)
                    }
                }
            }
            MessageMethod::SubscriptionAdd => {
                match event_sender {
                    Some(event_sender) => {
                        match router
                            .exec_subscription_unsafe(
                                ctx,
                                self.key.0.clone(),
                                self.key.1.unwrap_or(Value::Null),
                                self.key.2.unwrap_or(Value::Null),
                            )
                            .await
                        {
                            Ok(mut result) => {
                                let event_sender = event_sender.clone();
                                spawn(async move {
                                    while let Some(msg) = result.next().await {
                                        match msg {
                                            Ok(msg) => {
                                                if let Err(_) = event_sender.send(Response::Event(
                                                    EventResult {
                                                        key: self.key.0.clone(),
                                                        result: msg,
                                                    },
                                                )) {
                                                    println!("ERROR SENDING MESSAGE!"); // TODO: Error handling here
                                                    return;
                                                }
                                            }
                                            Err(_) => {
                                                println!("ERROR GETTING MESSAGE!"); // TODO: Error handling here
                                                return;
                                            }
                                        }
                                    }
                                });
                                Response::None
                            }
                            Err(err) => {
                                println!("Error: {}", err); // TODO: Proper error handling
                                Response::Response(ResponseResult::Error)
                            }
                        }
                    }
                    None => {
                        println!("Error: Can't add subscription without event sender"); // TODO: Proper error handling
                        Response::Response(ResponseResult::Error)
                    }
                }
            }
            MessageMethod::SubscriptionRemove => {
                unimplemented!(); // TODO: Make this work
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Response {
    Event(EventResult),
    Response(ResponseResult),
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventResult {
    key: String,
    result: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResponseResult {
    Success { id: Option<String>, result: Value },
    Error, // TODO: Make events work
}
