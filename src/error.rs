#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error deserializing procedure arguments: {0}")]
    ErrDeserializingArg(serde_json::Error),
    #[error("error serializing procedure result: {0}")]
    ErrSerializingArg(serde_json::Error),
}
