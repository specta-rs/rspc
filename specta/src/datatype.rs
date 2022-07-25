use crate::{EnumType, ObjectType, TupleType};

#[derive(Debug, Clone)]
pub enum DataType {
    // Always inlined
    Primitive(PrimitiveType),
    List(Box<DataType>),
    Nullable(Box<DataType>),
    // Can be exported
    Tuple(TupleType),
    Object(ObjectType),
    Enum(EnumType),
}

impl DataType {
    pub fn is_inline(&self) -> bool {
        match self {
            Self::Object(ObjectType { inline, .. })
            | Self::Enum(EnumType { inline, .. })
            | Self::Tuple(TupleType { inline, .. }) => *inline,
            Self::Nullable(typ) | Self::List(typ) => typ.is_inline(),
            _ => false,
        }
    }

    pub fn force_inline(&mut self) {
        match self {
            Self::Object(ObjectType { inline, .. })
            | Self::Enum(EnumType { inline, .. })
            | Self::Tuple(TupleType { inline, .. }) => *inline = true,
            Self::Nullable(typ) | Self::List(typ) => typ.force_inline(),
            _ => {}
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
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
    Path,
    PathBuf,
}
