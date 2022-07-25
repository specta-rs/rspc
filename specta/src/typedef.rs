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
        name: String,
        inline: bool,
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
        repr: EnumRepr
    },
}

impl BodyDefinition {
    pub fn is_inline(&self) -> bool {
        match self {
            Self::Object { inline, .. } | Self::Enum { inline, .. } => *inline,
            _ => false,
        }
    }

    pub fn force_inline(&mut self) {
        match self {
            Self::Object { inline, .. } | Self::Enum { inline, .. } | Self::Tuple { inline, .. } => *inline = true,
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
    Untagged
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
