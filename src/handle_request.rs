use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::{EventResult, Request, Response, ResponseResult, Router, StreamOrValue};

impl Request {
    pub async fn handle<TCtx, TMeta>(
        self,
        ctx: TCtx,
        router: &Arc<Router<TCtx, TMeta>>,
        event_sender: Option<&UnboundedSender<Response>>,
    ) -> Response
    where
        TCtx: Send + 'static,
        TMeta: Send + Sync + 'static,
    {
        match router.exec(ctx, self.operation, self.key.clone()).await {
            Ok(result) => match result {
                StreamOrValue::Stream(mut stream) => {
                    match event_sender {
                        Some(event_sender) => {
                            let event_sender = event_sender.clone();
                            let key = self.key.0.clone();
                            tokio::spawn(async move {
                                while let Some(msg) = stream.next().await {
                                    match msg {
                                        Ok(msg) => {
                                            if let Err(_) =
                                                event_sender.send(Response::Event(EventResult {
                                                    key: key.clone(),
                                                    result: msg,
                                                }))
                                            {
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
                        None => {
                            println!("Error: Can't add subscription without event sender"); // TODO: Proper error handling
                            Response::Response(ResponseResult::Error)
                        }
                    }
                }
                StreamOrValue::Value(v) => Response::Response(ResponseResult::Success {
                    id: self.id,
                    result: v,
                }),
            },
            Err(err) => {
                println!("Error: {}", err); // TODO: Proper error handling
                Response::Response(ResponseResult::Error)
            }
        }
    }
}
