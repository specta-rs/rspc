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
    Primitive(&'static str),
    UnitTuple,
    Tuple(Vec<Typedef>),
    List(Box<Typedef>),
    Nullable(Box<Typedef>),
    Object(Vec<Field>),
    Enum(Vec<Typedef>),
}
