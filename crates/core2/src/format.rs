pub trait Format {
    type Result; // TODO: Should we keep result???
    type Serializer: TODOSerializer;

    fn serializer(&self) -> Self::Serializer;

    // fn into_result(ser: Self::Serializer) -> Self::Result;
}

// TODO: Rename
pub trait TODOSerializer: Send {
    fn serialize_str(&mut self, s: &str);

    // TODO: Finish this
}
