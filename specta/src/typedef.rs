use std::any::TypeId;

/// TODO
#[derive(Debug, Clone)]
pub struct Typedef {
    pub type_id: TypeId,
    pub body: BodyDefinition,
}

/// TODO
#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: String,
    pub ty: Typedef,
}

/// TODO
#[derive(Debug, Clone)]
pub enum BodyDefinition {
    // Always inlined
    Primitive(PrimitiveType),
    List(Box<Typedef>),
    Nullable(Box<Typedef>),
    // Can be exported
    Tuple {
        name: Option<String>,
        fields: Vec<Typedef>,
    },
    Object {
        name: String,
        fields: Vec<ObjectField>,
        inline: bool,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        inline: bool,
    },
}

impl BodyDefinition {
    pub fn is_inline(&self) -> bool {
        match self {
            Self::Object { inline, .. }
            | Self::Enum { inline, .. } => *inline,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RefType {
    Object,
    Enum,
}

#[derive(Debug, Clone)]
pub enum EnumVariant {
    Unit(String),
    Unnamed(String, BodyDefinition), // Tuple
    Named(String, BodyDefinition),   // Object
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
    PathBuf,
}
