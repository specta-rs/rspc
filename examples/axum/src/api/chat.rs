//! TODO: Document how this example works. Especially `name` and `Clone` semantics for websocket connections.

use std::sync::{Arc, Mutex, PoisonError};

use super::{BaseProcedure, Router};

use async_stream::stream;
use serde::Serialize;
use specta::Type;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Ctx {
    author: Arc<Mutex<String>>,
    chat: broadcast::Sender<Message>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            author: Arc::new(Mutex::new("Anonymous".into())),
            chat: broadcast::channel(100).0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct Message {
    author: String,
    content: String,
    // If `true` the message is only sent to the author
    private: bool,
}

pub fn mount() -> Router {
    Router::new()
        .procedure("setName", {
            <BaseProcedure>::builder().mutation(|ctx, name: String| async move {
                *ctx.chat
                    .author
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner) = name;

                Ok(())
            })
        })
        .procedure("send", {
            <BaseProcedure>::builder().query(|ctx, msg: String| async move {
                // TODO: Presumably the developer should be able to make this only work for websocket connection. `ctx.loopback` will break if not.

                let (msg, private) = match msg.as_str() {
                    "// ping" => ("pong".into(), true),
                    cmd if msg.starts_with("//") => {
                        (format!("The command '{}' is not valid!", &cmd[2..]), true)
                    }
                    msg => (msg.into(), false),
                };

                let author = ctx
                    .chat
                    .author
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone();

                ctx.chat
                    .chat
                    .send(Message {
                        author,
                        content: msg,
                        private,
                    })
                    .unwrap(); // TODO: Proper error handling

                Ok(())
            })
        })
        .procedure("subscribe", {
            <BaseProcedure>::builder().subscription(|ctx, _: ()| async move {
                Ok(stream! {
                    let mut chat = ctx.chat.chat.subscribe();
                    while let Ok(msg) = chat.recv().await {
                        yield Ok(msg); // TODO: error handling
                    }
                })
            })
        })
}
