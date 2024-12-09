use std::error;

use rspc_core::ResolverError;
use serde::Serialize;
use specta::Type;

pub trait Error: error::Error + Send + Serialize + Type + 'static {
    fn into_resolver_error(self) -> ResolverError;
}
