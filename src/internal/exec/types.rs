use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(specta::Type))]
pub struct Request {
    // pub id: RequestId,
    // #[serde(flatten)]
    // pub inner: RequestInner,
}
