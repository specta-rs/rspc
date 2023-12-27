use crate::TODOSerializer;

pub struct Serializer<'a> {
    serializer: &'a mut dyn TODOSerializer,
}

impl<'a> Serializer<'a> {
    // TODO: Should this be `async` so we can yield and reset the serializer state for multiple values???
    pub fn serialize<T: serde::Serialize>(self, value: &T) {
        // TODO: Properly hook this up with Serde

        self.serializer.serialize_str("Hello World");
    }

    // TODO: API for bytes
}
