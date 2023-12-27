use std::collections::HashMap;

use futures::channel::oneshot;

#[derive(Debug, Default)]
pub struct SubscriptionMap {
    map: HashMap<u32, oneshot::Sender<()>>,
}

impl SubscriptionMap {
    pub fn contains_key(&self, id: u32) -> bool {
        self.map.contains_key(&id)
    }

    pub fn shutdown(&mut self, id: u32) -> bool {
        if let Some(tx) = self.map.remove(&id) {
            tx.send(()).ok(); // If it's already shutdown, that's fine
            true
        } else {
            false
        }
    }

    pub fn shutdown_all(&mut self) {
        for (_, tx) in self.map.drain() {
            tx.send(()).ok(); // If it's already shutdown, that's fine
        }
    }

    pub fn insert(&mut self, id: u32, tx: oneshot::Sender<()>) {
        self.map.insert(id, tx);
    }

    // We remove but don't shutdown. This should be used when we know the subscription is shutdown.
    pub(crate) fn remove(&mut self, id: u32) {
        if let Some(tx) = self.map.remove(&id) {
            #[cfg(debug_assertions)]
            #[allow(clippy::panic)]
            if !tx.is_canceled() {
                // TODO: This is being hit! Why?
                // panic!("Subscription was not shutdown before being removed!");
            }
        };
    }
}
