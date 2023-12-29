use std::{marker::PhantomData, pin::Pin};

use crate::TODOSerializer;

pub struct Serializer<'a> {
    serializer: Pin<&'a mut dyn TODOSerializer>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Serializer<'a> {
    pub(crate) fn new(serializer: Pin<&'a mut dyn TODOSerializer>) -> Self {
        Self {
            serializer,
            phantom: PhantomData,
        }
    }

    // TODO: Should this be `async` so we can yield and reset the serializer state for multiple values???
    pub fn serialize<T: serde::Serialize>(self, value: &T) {
        value.serialize(SerdeInterop(self)).unwrap();
    }

    // TODO: API for bytes
}

struct SerdeInterop<'a>(Serializer<'a>);

impl<'a> serde::Serializer for SerdeInterop<'a> {
    type Ok = ();
    // TODO: Stop using `serde_value` and use internal stuff
    type Error = serde_value::SerializerError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.serializer.serialize_str(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeSeq for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeTuple for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeTupleStruct for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeTupleVariant for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeMap for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeStruct for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde::ser::SerializeStructVariant for SerdeInterop<'a> {
    type Ok = ();
    type Error = serde_value::SerializerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
