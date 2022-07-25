use std::any::TypeId;

#[derive(Debug, Clone)]
pub struct Typedef {
    pub type_id: TypeId,
    pub body: DataType,
}

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: String,
    pub ty: Typedef,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub enum DataType {
    // Always inlined
    Primitive(PrimitiveType),
    List(Box<Typedef>),
    Nullable(Box<Typedef>),
    // Can be exported
    Tuple(TupleType),
    Object(ObjectType),
    Enum(EnumType),
}

#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub inline: bool,
    pub repr: EnumRepr,
}
#[derive(Debug, Clone)]
pub struct ObjectType {
    pub name: String,
    pub fields: Vec<ObjectField>,
    pub inline: bool,
    pub tag: Option<String>,
}
#[derive(Debug, Clone)]
pub struct TupleType {
    pub name: String,
    pub inline: bool,
    pub fields: Vec<Typedef>,
}

impl DataType {
    pub fn is_inline(&self) -> bool {
        match self {
            Self::Object(ObjectType { inline, .. }) | Self::Enum(EnumType { inline, .. }) => {
                *inline
            }
            _ => false,
        }
    }

    pub fn force_inline(&mut self) {
        match self {
            Self::Object(ObjectType { inline, .. })
            | Self::Enum(EnumType { inline, .. })
            | Self::Tuple(TupleType { inline, .. }) => *inline = true,
            Self::Nullable(def) | Self::List(def) => def.body.force_inline(),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum EnumRepr {
    External,
    Internal { tag: String },
    Adjacent { tag: String, content: String },
    Untagged,
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(String),
    Unnamed(TupleType),
    Named(ObjectType),
}

impl EnumVariant {
    pub fn name(&self) -> &str {
        match self {
            Self::Unit(name) => name,
            Self::Unnamed(tuple_type) => &tuple_type.name,
            Self::Named(object_type) => &object_type.name,
        }
    }

    pub fn data_type(&self) -> DataType {
        match self {
            Self::Unit(_) => DataType::Primitive(PrimitiveType::Never),
            Self::Unnamed(tuple_type) => DataType::Tuple(tuple_type.clone()),
            Self::Named(object_type) => DataType::Object(object_type.clone()),
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
