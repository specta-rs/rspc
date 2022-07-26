use crate::DataType;

#[derive(Debug, Clone)]
pub struct TupleType {
    pub name: String,
    pub fields: Vec<DataType>,
}
