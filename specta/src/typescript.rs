use std::collections::HashSet;

use crate::{
    DataType, EnumRepr, EnumType, EnumVariant, ObjectField, ObjectType, PrimitiveType, TupleType,
    TypeDefs,
};

use super::Type;

pub fn ts_inline<T: Type>() -> String {
    to_ts_inline(&T::def(&mut TypeDefs::default()))
}

pub fn ts_ref<T: Type>() -> String {
    to_ts(&T::def(&mut TypeDefs::default()), false)
}

pub fn ts_export<T: Type>() -> Result<String, String> {
    to_ts_export(&T::def(&mut TypeDefs::default()))
}

pub fn to_ts_export(def: &DataType) -> Result<String, String> {
    let inline = to_ts_inline(&def);

    Ok(match &def {
        def if def.is_inline() => return Err(format!("Cannot export inline type {:?}", def)),
        DataType::Object(ObjectType { name, .. }) => {
            format!("export interface {name} {inline}")
        }
        DataType::Enum(EnumType { name, .. }) => {
            format!("export type {name} = {inline}")
        }
        DataType::Tuple(TupleType { name, .. }) => {
            format!("export type {name} = {inline}")
        }
        _ => return Err(format!("Type cannot be exported: {:?}", def)),
    })
}

/// Prints the type definition of the given type.
/// `field_inline` is necessary since the type may have been
/// made inline outside of the type definition.
pub fn to_ts(typ: &DataType, field_inline: bool) -> String {
    match &typ {
        DataType::Enum(EnumType { name, inline, .. })
        | DataType::Object(ObjectType { name, inline, .. })
            if !inline && !field_inline =>
        {
            name.to_string()
        }
        DataType::Tuple(TupleType { fields, .. }) if fields.len() == 1 => to_ts(&fields[0], false),
        DataType::Nullable(def) => format!("{} | null", to_ts(&def, field_inline)),
        DataType::List(def) => format!("Array<{}>", to_ts(&def, field_inline)),
        body => to_ts_inline(body),
    }
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

/// Prints the inline type of the given type.
/// Inlining is not applied to fields and variants.
pub fn to_ts_inline(typ: &DataType) -> String {
    match &typ {
        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => "number".into(),
        primitive_def!(i64 u64 i128 u128) => "bigint".into(),
        primitive_def!(String char Path PathBuf) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        primitive_def!(Never) => "never".into(),
        DataType::Nullable(def) => format!("{} | null", to_ts_inline(&def)),
        DataType::List(def) => format!("Array<{}>", to_ts_inline(&def)),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "null".to_string(),
            [ty] => to_ts(&ty, false),
            tys => format!(
                "[{}]",
                tys.iter()
                    .map(|ty| to_ts(&ty, false))
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
                        let typ = to_ts(&DataType::Tuple(tuple.clone()), false);

                        format!("{{ {tag}: \"{sanitised_name}\" }} & {typ}")
                    }
                    (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                        let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                        fields.extend(object_fields(&obj.fields));

                        format!("{{ {} }}", fields.join(", "))
                    }
                    (EnumRepr::External, EnumVariant::Unit(_)) => format!("\"{sanitised_name}\""),
                    (EnumRepr::External, v) => {
                        let ts_values = to_ts(&v.data_type(), false);

                        format!("{{ {sanitised_name}: {ts_values} }}")
                    }
                    (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                    (EnumRepr::Untagged, v) => to_ts(&v.data_type(), false),
                    (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                        format!("{{ {tag}: \"{sanitised_name}\" }}")
                    }
                    (EnumRepr::Adjacent { tag, content }, v) => {
                        let ts_values = to_ts(&v.data_type(), false);

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

            let (key, ty) = match field.optional {
                true => (
                    format!("{}?", field_name_safe),
                    match &field.ty {
                        DataType::Nullable(ty) => ty.as_ref(),
                        ty => &ty,
                    },
                ),
                false => (field_name_safe, &field.ty),
            };

            format!("{key}: {}", to_ts(&ty, field.inline))
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

/// Resolves dependencies of a type.
/// Ignores inlining as the type passed in is treated as the root type.
pub fn ts_dependencies(ty: &DataType) -> HashSet<&str> {
    match &ty {
        DataType::Primitive(_) => HashSet::new(),
        DataType::Nullable(def) | DataType::List(def) => ts_dependencies(&def),
        DataType::Object(obj) => obj
            .fields
            .iter()
            .flat_map(|field| ts_field_dependencies(&field.ty, field.inline))
            .collect(),
        DataType::Enum(e) => ts_enum_dependencies(e).into_iter().collect(),
        DataType::Tuple(tuple) => tuple
            .fields
            .iter()
            .flat_map(|f| ts_field_dependencies(&f, false))
            .collect(),
    }
}

/// Resolves dependencies of a type as if it is a dependency of some parent type.
/// Similar to ts_dependencies, but if the type is inlineable and inline is false
/// it will itself be a dependency, indicating that the parent type depends on it.
fn ts_field_dependencies(ty: &DataType, field_inline: bool) -> Vec<&str> {
    match &ty {
        DataType::Primitive(_) => vec![],
        DataType::Nullable(def) | DataType::List(def) => ts_field_dependencies(&def, field_inline),
        DataType::Object(obj) => match field_inline || obj.inline {
            true => obj
                .fields
                .iter()
                .flat_map(|obj_field| ts_field_dependencies(&obj_field.ty, obj_field.inline))
                .collect(),
            false => vec![obj.name.as_str()],
        },
        DataType::Enum(e) => match field_inline || e.inline {
            true => ts_enum_dependencies(e),
            false => vec![e.name.as_str()],
        },
        DataType::Tuple(tuple) => match field_inline || tuple.inline {
            true => tuple
                .fields
                .iter()
                .flat_map(|field| ts_field_dependencies(&field, field.is_inline()))
                .collect(),
            false => vec![tuple.name.as_str()],
        },
    }
}

fn ts_enum_dependencies(e: &EnumType) -> Vec<&str> {
    e.variants
        .iter()
        .flat_map(|v| match v {
            EnumVariant::Unit(_) => vec![],
            EnumVariant::Unnamed(tuple) => tuple
                .fields
                .iter()
                .flat_map(|f| ts_field_dependencies(&f, false))
                .collect(),
            EnumVariant::Named(obj) => obj
                .fields
                .iter()
                .flat_map(|f| ts_field_dependencies(&f.ty, false))
                .collect(),
        })
        .collect()
}
