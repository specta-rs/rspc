use crate::DataType;

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: String,
    pub ty: DataType,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    pub name: String,
    pub inline: bool,
    pub generics: Vec<Generic>,
    pub fields: Vec<ObjectField>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Generic {
    TypeParam { name: String, ty: DataType },
}
