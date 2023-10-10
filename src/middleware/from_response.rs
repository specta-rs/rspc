pub trait FromResponse {}

impl FromResponse for serde_json::Value {
    // TODO: Decoding from `rspc_core::Body`
}
