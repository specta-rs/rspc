use std::any::TypeId;

use crate::{DataType, ObjectType, TupleType};

#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub generics: Vec<&'static str>,
    pub repr: EnumRepr,
    pub type_id: TypeId,
}

impl EnumType {
    pub fn make_flattenable(&mut self) {
        let indexes = self
            .variants
            .iter()
            .filter(|v| match self.repr {
                EnumRepr::External => match v {
                    EnumVariant::Unnamed(v) if v.fields.len() == 1 => false,
                    EnumVariant::Named(_) => false,
                    _ => true,
                },
                EnumRepr::Untagged => match v {
                    EnumVariant::Unit(_) => false,
                    EnumVariant::Named(_) => false,
                    _ => true,
                },
                EnumRepr::Adjacent { .. } => false,
                EnumRepr::Internal { .. } => match v {
                    EnumVariant::Unit(_) => false,
                    EnumVariant::Named(_) => false,
                    _ => true,
                },
            })
            .enumerate()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        indexes.into_iter().rev().for_each(|i| {
            self.variants.remove(i);
        });
    }
}

impl PartialEq for EnumType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
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
            Self::Unit(_) => unreachable!("Unit enum variants have no type!"),
            Self::Unnamed(tuple_type) => DataType::Tuple(tuple_type.clone()),
            Self::Named(object_type) => DataType::Object(object_type.clone()),
        }
    }
}
