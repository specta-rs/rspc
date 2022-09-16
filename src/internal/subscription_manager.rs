use std::sync::Arc;
// use tokio::sync::{oneshot, Mutex};

/// is responsible for managing all of the subscriptions.
pub struct SubscriptionManager {
    // subscriptions: Mutex<
    //     HashMap<
    //         (), // (
    //             //     /* operation key */ String,
    //             //     /* method */ String,
    //             //     /* id */ String,
    //             // ),
    //             // oneshot::Sender<()>,
    //     >,
    // >,
}

impl SubscriptionManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            // subscriptions: Mutex::new(HashMap::new()),
        })
    }

    // pub fn register(&self, operation: String, method: String, id: String) -> oneshot::Receiver<()> {
    //     let (tx, rx) = oneshot::channel();
    //     self.subscriptions
    //         .lock()
    //         .unwrap()
    //         .insert((operation, method, id), tx);
    //     rx
    // }

    // pub fn unregister(&self, operation: String, method: String, id: String) -> Result<(), ()> {
    //     self.subscriptions
    //         .lock()
    //         .unwrap()
    //         .remove(&(operation, method, id))
    //         .ok_or(())?
    //         .send(())
    //         .map(|_| ())
    // }
}
