use crate::{DataType, GenericType};

#[derive(Debug, Clone)]
pub struct TupleType {
    pub name: String,
    pub fields: Vec<DataType>,
    pub generics: Vec<GenericType>,
}
