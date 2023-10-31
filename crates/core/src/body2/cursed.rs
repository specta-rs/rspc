use std::{
    cell::Cell,
    future::poll_fn,
    task::{Poll, Waker},
};

use crate::Body;

// TODO: Make this private
pub enum YieldMsg {
    YieldBody,
    YieldBodyResult(serde_json::Value),
}

thread_local! {
    // TODO: Make this private
    pub static CURSED_OP: Cell<Option<YieldMsg>> = const { Cell::new(None) };
}

// TODO: Make private
pub async fn inner() -> Body {
    let mut state = false;
    poll_fn(|_| match state {
        false => {
            CURSED_OP.set(Some(YieldMsg::YieldBody));
            state = true;
            return Poll::Pending;
        }
        true => {
            let y = CURSED_OP
                .take()
                .expect("Expected response from outer future!");
            return Poll::Ready(match y {
                YieldMsg::YieldBody => unreachable!(),
                YieldMsg::YieldBodyResult(body) => Body::Value(body),
            });
        }
    })
    .await
}

// TODO: Use this instead
// // Called on `Poll::Pending` from inner
// pub fn outer(waker: &Waker) {
//     if let Some(op) = CURSED_OP.take() {
//         match op {
//             YieldMsg::YieldBody => {
//                 // TODO: Get proper value
//                 CURSED_OP.set(Some(YieldMsg::YieldBodyResult(serde_json::Value::Null)));
//                 waker.wake_by_ref();
//             }
//             YieldMsg::YieldBodyResult(_) => unreachable!(),
//         }
//     }
// }
