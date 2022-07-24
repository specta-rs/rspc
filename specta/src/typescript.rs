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

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(BodyDefinition::Primitive(stringify!($t)))|+
    }
}

pub fn get_ts_type(def: &Typedef) -> Result<String, String> {
    match &def.body {
        primitive_def!(i8 i16 u16 i32 u32 f32 f64 usize isize) => Ok("number".into()),
        primitive_def!(i64 u64 i128 u128) => Ok("bigint".into()),
        primitive_def!(String char Path PathBuf) => Ok("string".into()),
        primitive_def!(bool) => Ok("boolean".into()),
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
