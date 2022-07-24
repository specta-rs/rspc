use std::any::TypeId;

/// TODO
#[derive(Debug, Clone)]
pub struct Typedef {
    pub name: String,
    pub primitive: bool,
    pub type_id: TypeId,
    pub body: BodyDefinition,
}

/// TODO
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Typedef,
}

/// TODO
#[derive(Debug, Clone)]
pub enum BodyDefinition {
    Primitive(PrimitiveType),
    Tuple(Vec<Typedef>),
    List(Box<Typedef>),
    Nullable(Box<Typedef>),
    Object(Vec<Field>),
    Enum(Vec<EnumVariant>),
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(String),
    Unnamed(String, BodyDefinition), // Tuple
    Named(String, BodyDefinition), // Object
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
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
    Path,
    PathBuf
}
