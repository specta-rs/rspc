use std::any::TypeId;

use crate::{EnumType, ObjectType};

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    // Always inlined
    Any,
    Primitive(PrimitiveType),
    List(Box<DataType>),
    Nullable(Box<DataType>),
    Record(Box<(DataType, DataType)>),
    Tuple(TupleType),
    // Reference types
    Object(ObjectType),
    Enum(EnumType),
    // A reference type that has already been defined
    Reference {
        name: String,
        generics: Vec<DataType>,
        type_id: TypeId,
    },
    Generic(String),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    Never,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    bool,
    char,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleType {
    pub name: String,
    pub fields: Vec<DataType>,
    pub generics: Vec<&'static str>,
}
