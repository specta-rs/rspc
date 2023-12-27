pub struct Serializer<'a> {
    serializer: &'a mut (dyn erased_serde::Serializer + Send),
}

impl<'a> Serializer<'a> {}

// TODO: How could this serialize bytes/files

// pub struct BytesSerializer {}

// pub struct ValueSerializer {}
