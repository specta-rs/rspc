use std::error;

use serde::Serialize;
use specta::Type;

pub trait Error: error::Error + Send + Serialize + Type + 'static {
    // Warning: Returning > 400 will fallback to `500`. As redirects would be invalid and `200` would break matching.
    fn status(&self) -> u16 {
        500
    }
}

// impl Error for rspc_core:: {}
