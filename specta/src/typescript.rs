use crate::{BodyDefinition, TypeDefs, Typedef};

use super::Type;

pub fn typescript_export<T: Type>() -> Result<String, String> {
    to_ts(T::def(&mut TypeDefs::default()))
}

pub fn to_ts(def: Typedef) -> Result<String, String> {
    if def.primitive {
        return Err("Primitive types can be exported!".to_string());
    }

    if matches!(def.body, BodyDefinition::Object(_)) {
        Ok(format!(
            "export interface {} {};",
            def.name,
            get_ts_type(&def)?
        ))
    } else {
        Ok(format!(
            "export type {} = {};",
            def.name,
            get_ts_type(&def)?
        ))
    }
}

pub fn get_ts_type(def: &Typedef) -> Result<String, String> {
    match &def.body {
        BodyDefinition::Primitive(stringify!(i8))
        | BodyDefinition::Primitive(stringify!(u8))
        | BodyDefinition::Primitive(stringify!(i16))
        | BodyDefinition::Primitive(stringify!(u16))
        | BodyDefinition::Primitive(stringify!(i32))
        | BodyDefinition::Primitive(stringify!(u32))
        | BodyDefinition::Primitive(stringify!(f32))
        | BodyDefinition::Primitive(stringify!(f64))
        | BodyDefinition::Primitive(stringify!(usize))
        | BodyDefinition::Primitive(stringify!(isize)) => Ok("number".into()),
        BodyDefinition::Primitive(stringify!(i64))
        | BodyDefinition::Primitive(stringify!(u64))
        | BodyDefinition::Primitive(stringify!(i128))
        | BodyDefinition::Primitive(stringify!(u128)) => Ok("bigint".into()),
        BodyDefinition::Primitive(stringify!(bool)) => Ok("boolean".into()),
        BodyDefinition::Primitive(stringify!(String))
        | BodyDefinition::Primitive(stringify!(char))
        | BodyDefinition::Primitive(stringify!(Path))
        | BodyDefinition::Primitive(stringify!(PathBuf)) => Ok("string".into()),
        BodyDefinition::UnitTuple => Ok("null".into()),
        BodyDefinition::Tuple(def) => Ok(format!(
            "[{}]",
            def.into_iter()
                .map(|v| get_ts_type(&v))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        )),
        BodyDefinition::List(def) => Ok(format!("{}[]", get_ts_type(&*def)?)),
        BodyDefinition::Nullable(def) => Ok(format!("{} | null", get_ts_type(&*def)?)),
        BodyDefinition::Object(def) => Ok(format!(
            "{{ {} }}",
            def.into_iter()
                .map(|field| get_ts_type(&field.ty).map(|v| format!("{}: {}", field.name, v)))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        )),
        BodyDefinition::Enum(def) => Ok(format!(
            "{}",
            def.into_iter()
                .map(|v| get_ts_type(&v))
                .collect::<Result<Vec<_>, _>>()?
                .join(" | ")
        )),
        _ => Err(format!(
            "Could not convert type '{}' to Typescript type!",
            def.name
        )),
    }
}
