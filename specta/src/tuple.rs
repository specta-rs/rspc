use std::any::TypeId;

use crate::DataType;

#[derive(Debug, Clone)]
pub struct TupleType {
    pub name: String,
    pub id: TypeId,
    pub fields: Vec<DataType>,
}
