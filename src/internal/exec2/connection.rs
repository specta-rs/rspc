use std::collections::HashMap;

/// A connection holds the shared state for a specific connection.
///
/// A connection would be a single websocket connection or page load in a Tauri context.
pub struct Connection {
    pub(crate) subscriptions: HashMap<u32, ()>,
}

// TODO: Debug impl

impl Connection {
    pub fn new() -> Self {
        Self {
            subscriptions: Default::default(),
        }
    }

    // pub fn reset() {
    //     todo!();
    // }
}

// impl Drop for Connection {
//     fn drop(&mut self) {
//         // TODO: Queue all subscriptions to be dropped
//     }
// }

// // TODO: Break into another file???
// pub struct ShutdownSignal {
//     signal: AtomicBool,
//     waker: Mutex<Option<Waker>>,
// }
