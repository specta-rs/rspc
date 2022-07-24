use crate::{BodyDefinition, EnumVariant, PrimitiveType, TypeDefs, Typedef};

use super::Type;

pub fn ts_definition<T: Type>() -> String {
    let def = T::def(&mut TypeDefs::default()).body;
    to_ts_definition(&def)
}

pub fn ts_export<T: Type>() -> Result<String, String> {
    to_ts_export(T::def(&mut TypeDefs::default()))
}

fn to_ts_export(def: Typedef) -> Result<String, String> {
    let anon_typ = to_ts_definition(&def.body);

    Ok(match &def.body {
        body if body.is_inline() => return Err(format!("Cannot export inline type {:?}", def)),
        BodyDefinition::Object { name, .. } => {
            format!("export interface {name} {anon_typ}")
        }
        BodyDefinition::Enum { name, .. } => {
            format!("export type {name} = {anon_typ}")
        }
        BodyDefinition::Tuple { name, .. } => match name {
            Some(name) => format!("export type {name} = {anon_typ}"),
            None => return Err(format!("Cannot export anonymous tuple: {:?}", def)),
        },
        _ => return Err(format!("Type cannot be exported: {:?}", def)),
    })
}

fn to_ts_reference(body: &BodyDefinition) -> String {
    match &body {
        BodyDefinition::Enum { name, inline, .. } | BodyDefinition::Object { name, inline, .. }
            if !inline =>
        {
            name.to_string()
        }
        body => to_ts_definition(body),
    }
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(BodyDefinition::Primitive(PrimitiveType::$t))|+
    }
}

fn to_ts_definition(body: &BodyDefinition) -> String {
    match &body {
        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => "number".into(),
        primitive_def!(i64 u64 i128 u128) => "bigint".into(),
        primitive_def!(String char Path PathBuf) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        BodyDefinition::Tuple { fields, .. } => match &fields[..] {
            [] => "null".to_string(),
            [item] => to_ts_reference(&item.body),
            items => format!(
                "[{}]",
                items
                    .iter()
                    .map(|v| to_ts_reference(&v.body))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        BodyDefinition::List(def) => format!("{}[]", to_ts_reference(&def.body)),
        BodyDefinition::Nullable(def) => format!("{} | null", to_ts_reference(&def.body)),
        BodyDefinition::Object { fields, .. } => match &fields[..] {
            [] => "null".to_string(),
            items => format!(
                "{{ {} }}",
                items
                    .iter()
                    .map(|field| format!("{}: {}", field.name, to_ts_reference(&field.ty.body)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
        BodyDefinition::Enum { variants, .. } => variants
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

    let values_ts = to_ts_reference(values);

    format!("{{ {name}: {values_ts} }}")
}
