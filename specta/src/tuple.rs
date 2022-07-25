use std::any::TypeId;

use crate::DataType;

#[derive(Debug, Clone)]
pub struct TupleType {
    pub name: String,
    pub id: TypeId,
    pub inline: bool,
    pub fields: Vec<DataType>,
}
