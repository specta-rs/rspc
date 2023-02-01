use std::{any::TypeId, collections::BTreeMap};

mod r#enum;
mod object;

pub use object::*;
pub use r#enum::*;

use crate::{ImplLocation, TypeSid};

/// A map of type definitions
pub type TypeDefs = BTreeMap<&'static str, DataTypeExt>;

/// arguments for [Type::inline](crate::Type::inline), [Type::reference](crate::Type::reference) and [Type::definition](crate::Type::definition).
pub struct DefOpts<'a> {
    /// is the parent type inlined?
    pub parent_inline: bool,
    /// a map of types which have been visited. This prevents stack overflows when a type references itself and also allows the caller to get a list of all types in the "schema".
    pub type_map: &'a mut TypeDefs,
}

/// a wrapper around data type that can store the type erased data type and the doc comments on the type
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct DataTypeExt {
    pub name: &'static str,
    pub comments: &'static [&'static str],
    pub sid: TypeSid,
    pub impl_location: ImplLocation,
    pub inner: DataType,
}

/// this is used internally to represent the types.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum DataType {
    // Always inlined
    Any,
    Primitive(PrimitiveType),
    Literal(LiteralType),
    List(Box<DataType>),
    Nullable(Box<DataType>),
    Record(Box<(DataType, DataType)>),
    Tuple(TupleType),
    // Reference types
    Object(ObjectType),
    Enum(EnumType),
    // A reference type that has already been defined
    Reference {
        name: &'static str,
        generics: Vec<DataType>,
        type_id: TypeId,
    },
    Generic(GenericType),
    Placeholder,
}

/// this is used internally to represent the types.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct GenericType(pub &'static str);

/// this is used internally to represent the types.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum PrimitiveType {
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

impl PrimitiveType {
    /// Converts a `PrimitiveType` into a Rust code string.
    pub fn to_rust_str(&self) -> &'static str {
        match self {
            Self::i8 => "i8",
            Self::i16 => "i16",
            Self::i32 => "i32",
            Self::i64 => "i64",
            Self::i128 => "i128",
            Self::isize => "isize",
            Self::u8 => "u8",
            Self::u16 => "u16",
            Self::u32 => "u32",
            Self::u64 => "u64",
            Self::u128 => "u128",
            Self::usize => "usize",
            Self::f32 => "f32",
            Self::f64 => "f64",
            Self::bool => "bool",
            Self::char => "char",
            Self::String => "String",
        }
    }
}

/// this is used internally to represent the types.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct TupleType {
    pub name: &'static str,
    pub fields: Vec<DataType>,
    pub generics: Vec<&'static str>,
}

/// this is used internally to represent the types.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum LiteralType {
    i8(i8),
    i16(i16),
    i32(i32),
    u8(u8),
    u16(u16),
    u32(u32),
    f32(f32),
    f64(f64),
    bool(bool),
    String(String),
    None,
}

impl From<PrimitiveType> for DataType {
    fn from(t: PrimitiveType) -> Self {
        Self::Primitive(t)
    }
}

impl From<LiteralType> for DataType {
    fn from(t: LiteralType) -> Self {
        Self::Literal(t)
    }
}

impl From<ObjectType> for DataType {
    fn from(t: ObjectType) -> Self {
        Self::Object(t)
    }
}

impl From<EnumType> for DataType {
    fn from(t: EnumType) -> Self {
        Self::Enum(t)
    }
}

impl From<GenericType> for DataType {
    fn from(t: GenericType) -> Self {
        Self::Generic(t)
    }
}

impl From<TupleType> for DataType {
    fn from(t: TupleType) -> Self {
        Self::Tuple(t)
    }
}

impl<T: Into<DataType> + 'static> From<Vec<T>> for DataType {
    fn from(t: Vec<T>) -> Self {
        Self::Enum(EnumType {
            name: "",
            variants: t
                .into_iter()
                .map(|t| -> EnumVariant {
                    EnumVariant::Unnamed(TupleType {
                        name: "",
                        fields: vec![t.into()],
                        generics: vec![],
                    })
                })
                .collect(),
            generics: vec![],
            repr: EnumRepr::Untagged,
            type_id: TypeId::of::<Vec<T>>(),
        })
    }
}

impl<T: Into<DataType> + 'static> From<Option<T>> for DataType {
    fn from(t: Option<T>) -> Self {
        t.map(Into::into)
            .unwrap_or_else(|| LiteralType::None.into())
    }
}

impl<'a> From<&'a str> for DataType {
    fn from(t: &'a str) -> Self {
        LiteralType::String(t.to_string()).into()
    }
}

impl From<String> for DataType {
    fn from(t: String) -> Self {
        LiteralType::String(t).into()
    }
}

impl From<DataTypeExt> for DataType {
    fn from(value: DataTypeExt) -> Self {
        value.inner
    }
}
