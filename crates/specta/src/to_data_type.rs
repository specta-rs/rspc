use std::any::TypeId;

use crate::{
    DataType, EnumRepr, EnumType, EnumVariant, LiteralType, ObjectType, PrimitiveType, TupleType,
};

pub trait ToDataType {
    fn to_data_type(self) -> DataType;
}

impl ToDataType for DataType {
    fn to_data_type(self) -> DataType {
        self.clone()
    }
}

impl ToDataType for PrimitiveType {
    fn to_data_type(self) -> DataType {
        DataType::Primitive(self.clone())
    }
}

impl ToDataType for LiteralType {
    fn to_data_type(self) -> DataType {
        DataType::Literal(self.clone())
    }
}

impl ToDataType for ObjectType {
    fn to_data_type(self) -> DataType {
        DataType::Object(self)
    }
}

impl ToDataType for EnumType {
    fn to_data_type(self) -> DataType {
        DataType::Enum(self)
    }
}

impl<T: ToDataType + 'static> ToDataType for Vec<T> {
    fn to_data_type(self) -> DataType {
        DataType::Enum(EnumType {
            name: "".to_string(),
            variants: self
                .into_iter()
                .map(|t| -> EnumVariant {
                    EnumVariant::Unnamed(TupleType {
                        name: "".to_string(),
                        fields: vec![t.to_data_type()],
                        generics: vec![],
                    })
                })
                .collect(),
            generics: vec![],
            repr: EnumRepr::Untagged,
            type_id: TypeId::of::<Self>(),
        })
    }
}

impl<T: ToDataType + 'static> ToDataType for Option<T> {
    fn to_data_type(self) -> DataType {
        self.map(ToDataType::to_data_type)
            .unwrap_or(LiteralType::None.to_data_type())
    }
}

impl<'a> ToDataType for &'a str {
    fn to_data_type(self) -> DataType {
        LiteralType::String(self.to_string()).to_data_type()
    }
}

impl ToDataType for String {
    fn to_data_type(self) -> DataType {
        LiteralType::String(self).to_data_type()
    }
}
