use crate::{
    DataType, DefOpts, EnumRepr, EnumType, EnumVariant, ObjectField, ObjectType, PrimitiveType,
    TupleType, TypeDefs,
};

use crate::Type;

pub fn ts_inline_ref<T: Type>(_t: &T) -> String {
    ts_inline::<T>()
}

pub fn ts_inline<T: Type>() -> String {
    to_ts(&T::inline(
        DefOpts {
            parent_inline: true,
            type_map: &mut TypeDefs::new(),
        },
        &[],
    ))
}

pub fn ts_ref_ref<T: Type>(_t: &T) -> String {
    ts_inline::<T>()
}

pub fn ts_ref<T: Type>() -> String {
    to_ts(&T::reference(
        DefOpts {
            parent_inline: false,
            type_map: &mut TypeDefs::new(),
        },
        &[],
    ))
}

pub fn ts_export_ref<T: Type>(_t: &T) -> String {
    ts_inline::<T>()
}

pub fn ts_export<T: Type>() -> Result<String, String> {
    to_ts_export(&T::inline(
        DefOpts {
            parent_inline: true,
            type_map: &mut TypeDefs::default(),
        },
        &[],
    ))
}

pub fn to_ts_export(def: &DataType) -> Result<String, String> {
    let inline_ts = to_ts(def);

    Ok(match &def {
        // Named struct
        DataType::Object(ObjectType {
            name,
            generics,
            fields,
            ..
        }) => match fields.len() {
            0 => format!("export type {name} = {inline_ts}"),
            _ => {
                let generics = match generics.len() {
                    0 => "".into(),
                    _ => format!("<{}>", generics.to_vec().join(", ")),
                };

                format!("export interface {name}{generics} {inline_ts}")
            }
        },
        // Enum
        DataType::Enum(EnumType { name, generics, .. }) => {
            let generics = match generics.len() {
                0 => "".into(),
                _ => format!("<{}>", generics.to_vec().join(", ")),
            };

            format!("export type {name}{generics} = {inline_ts}")
        }
        // Unnamed struct
        DataType::Tuple(TupleType { name, .. }) => {
            format!("export type {name} = {inline_ts}")
        }
        _ => return Err(format!("Type cannot be exported: {:?}", def)),
    })
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

pub fn to_ts(typ: &DataType) -> String {
    match &typ {
        DataType::Any => "any".into(),
        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => "number".into(),
        primitive_def!(i64 u64 i128 u128) => "bigint".into(),
        primitive_def!(String char) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        primitive_def!(Never) => "never".into(),
        DataType::Nullable(def) => format!("{} | null", to_ts(def)),
        DataType::Record(def) => {
            format!("Record<{}, {}>", to_ts(&def.0), to_ts(&def.1))
        }
        DataType::List(def) => format!("Array<{}>", to_ts(def)),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "null".to_string(),
            [ty] => to_ts(ty),
            tys => format!("[{}]", tys.iter().map(to_ts).collect::<Vec<_>>().join(", ")),
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
        DataType::Enum(EnumType { variants, repr, .. }) => match &variants[..] {
            [] => "never".to_string(),
            variants => variants
                .iter()
                .map(|variant| {
                    let sanitised_name = sanitise_name(variant.name());

                    match (repr, variant) {
                        (EnumRepr::Internal { tag }, EnumVariant::Unit(_)) => {
                            format!("{{ {tag}: \"{sanitised_name}\" }}")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                            let typ = to_ts(&DataType::Tuple(tuple.clone()));

                            format!("{{ {tag}: \"{sanitised_name}\" }} & {typ}")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                            let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                            fields.extend(object_fields(&obj.fields));

                            format!("{{ {} }}", fields.join(", "))
                        }
                        (EnumRepr::External, EnumVariant::Unit(_)) => {
                            format!("\"{sanitised_name}\"")
                        }
                        (EnumRepr::External, v) => {
                            let ts_values = to_ts(&v.data_type());

                            format!("{{ {sanitised_name}: {ts_values} }}")
                        }
                        (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                        (EnumRepr::Untagged, v) => to_ts(&v.data_type()),
                        (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                            format!("{{ {tag}: \"{sanitised_name}\" }}")
                        }
                        (EnumRepr::Adjacent { tag, content }, v) => {
                            let ts_values = to_ts(&v.data_type());

                            format!("{{ {tag}: \"{sanitised_name}\", {content}: {ts_values} }}")
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" | "),
        },
        DataType::Reference { name, generics, .. } => match &generics[..] {
            [] => name.to_string(),
            generics => {
                let generics = generics.iter().map(to_ts).collect::<Vec<_>>().join(", ");

                format!("{name}<{generics}>")
            }
        },
        DataType::Generic(ident) => ident.to_string(),
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
                        ty => ty,
                    },
                ),
                false => (field_name_safe, &field.ty),
            };

            format!("{key}: {}", to_ts(ty))
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
