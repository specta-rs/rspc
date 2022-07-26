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
    Reference(String),
}

impl DataType {
    pub fn force_inline(&mut self) {
        match self {
            DataType::Object(object) => object.inline = true,
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
