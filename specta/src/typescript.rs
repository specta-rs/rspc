use crate::{
    DataType, EnumRepr, EnumType, EnumVariant, ObjectField, ObjectType, PrimitiveType, TupleType,
    TypeDefs, Typedef,
};

use super::Type;

pub fn ts_definition<T: Type>() -> String {
    let def = T::def(&mut TypeDefs::default()).body;
    to_ts_definition(&def)
}

pub fn ts_export<T: Type>() -> Result<String, String> {
    to_ts_export(T::def(&mut TypeDefs::default()))
}

pub fn to_ts_export(def: Typedef) -> Result<String, String> {
    let anon_typ = to_ts_definition(&def.body);

    Ok(match &def.body {
        body if body.is_inline() => return Err(format!("Cannot export inline type {:?}", def)),
        DataType::Object(ObjectType { name, .. }) => {
            format!("export interface {name} {anon_typ}")
        }
        DataType::Enum(EnumType { name, .. }) => {
            format!("export type {name} = {anon_typ}")
        }
        DataType::Tuple(TupleType { name, .. }) => {
            format!("export type {name} = {anon_typ}")
        }
        _ => return Err(format!("Type cannot be exported: {:?}", def)),
    })
}

pub fn to_ts_reference(typ: &DataType) -> String {
    match &typ {
        DataType::Enum(EnumType { name, inline, .. })
        | DataType::Object(ObjectType { name, inline, .. })
            if !inline =>
        {
            name.to_string()
        }
        DataType::Tuple(TupleType { fields, .. }) if fields.len() == 1 => {
            to_ts_reference(&fields[0].body)
        }
        body => to_ts_definition(body),
    }
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

pub fn to_ts_definition(body: &DataType) -> String {
    match &body {
        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => "number".into(),
        primitive_def!(i64 u64 i128 u128) => "bigint".into(),
        primitive_def!(String char Path PathBuf) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        primitive_def!(Never) => "never".into(),
        DataType::Nullable(def) => format!("{} | null", to_ts_reference(&def.body)),
        DataType::List(def) => format!("Array<{}>", to_ts_reference(&def.body)),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
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
        DataType::Object(ObjectType {
            fields, tag, name, ..
        }) => match &fields[..] {
            [] => "null".to_string(),
            fields => {
                let mut out = match tag {
                    Some(tag) => vec![format!("{tag}: \"{name}\"")],
                    None => vec![],
                };

                let field_defs = object_fields(fields);

                out.extend(field_defs);

                format!("{{ {} }}", out.join(", "))
            }
        },
        DataType::Enum(EnumType { variants, repr, .. }) => variants
            .iter()
            .map(|variant| {
                let sanitised_name = sanitise_name(variant.name());

                match (repr, variant) {
                    (EnumRepr::Internal { tag }, EnumVariant::Unit(_)) => {
                        format!("{{ {tag}: \"{sanitised_name}\" }}")
                    }
                    (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                        let typ = to_ts_reference(&DataType::Tuple(tuple.clone()));

                        format!("{{ {tag}: \"{sanitised_name}\" }} & {typ}")
                    }
                    (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                        let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                        fields.extend(object_fields(&obj.fields));

                        format!("{{ {} }}", fields.join(", "))
                    }
                    (EnumRepr::External, EnumVariant::Unit(_)) => format!("\"{sanitised_name}\""),
                    (EnumRepr::External, v) => {
                        let ts_values = to_ts_reference(&v.data_type());

                        format!("{{ {sanitised_name}: {ts_values} }}")
                    }
                    (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                    (EnumRepr::Untagged, v) => to_ts_reference(&v.data_type()),
                    (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                        format!("{{ {tag}: \"{sanitised_name}\" }}")
                    }
                    (EnumRepr::Adjacent { tag, content }, v) => {
                        let ts_values = to_ts_reference(&v.data_type());

                        format!("{{ {tag}: \"{sanitised_name}\", {content}: {ts_values} }}")
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" | "),
    }
}

pub fn object_fields(fields: &[ObjectField]) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let field_name_safe = sanitise_name(&field.name);

            let (key, body) = match field.optional {
                true => (
                    format!("{}?", field_name_safe),
                    match &field.ty.body {
                        DataType::Nullable(def) => &def.body,
                        body => &body,
                    },
                ),
                false => (field_name_safe, &field.ty.body),
            };

            format!("{key}: {}", to_ts_reference(&body))
        })
        .collect::<Vec<_>>()
}

pub fn sanitise_name(value: &str) -> String {
    let valid = value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        && value
            .chars()
            .next()
            .map(|first| !first.is_numeric())
            .unwrap_or(true);
    if !valid {
        format!(r#""{value}""#)
    } else {
        value.to_string()
    }
}
