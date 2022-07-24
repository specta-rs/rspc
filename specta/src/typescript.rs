use crate::{BodyDefinition, EnumVariant, PrimitiveType, TypeDefs, Typedef};

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
            get_ts_type(&def.body)
        ))
    } else {
        Ok(format!(
            "export type {} = {};",
            def.name,
            get_ts_type(&def.body)
        ))
    }
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(BodyDefinition::Primitive(PrimitiveType::$t))|+
    }
}

pub fn get_ts_type(body: &BodyDefinition) -> String {
    match &body {
        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => "number".into(),
        primitive_def!(i64 u64 i128 u128) => "bigint".into(),
        primitive_def!(String char Path PathBuf) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        BodyDefinition::Tuple(def) => match &def[..] {
            [] => "null".to_string(),
            [item] => get_ts_type(&item.body),
            items => format!(
                "[{}]",
                items
                    .iter()
                    .map(|v| get_ts_type(&v.body))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        BodyDefinition::List(def) => format!("{}[]", get_ts_type(&def.body)),
        BodyDefinition::Nullable(def) => format!("{} | null", get_ts_type(&def.body)),
        BodyDefinition::Object(def) => match &def[..] {
            [] => "null".to_string(),
            items => format!(
                "{{ {} }}",
                items
                    .iter()
                    .map(|field| format!("{}: {}", field.name, get_ts_type(&field.ty.body)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        BodyDefinition::Enum(def) => def
            .iter()
            .map(|v| get_enum_ts_type(v))
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

pub fn get_enum_ts_type(def: &EnumVariant) -> String {
    let (name, values) = match &def {
        EnumVariant::Unit(name) => return format!(r#""{name}""#),
        EnumVariant::Unnamed(name, fields) => (name, fields),
        EnumVariant::Named(name, object) => (name, object),
    };

    let values_ts = get_ts_type(values);

    format!("{{ {name}: {values_ts} }}")
}
