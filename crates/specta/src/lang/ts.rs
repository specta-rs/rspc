use crate::*;

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string with an export.
/// Eg. `export type Foo = { demo: string; };`
pub fn export<T: Type>() -> Result<String, String> {
    export_datatype(&T::definition(DefOpts {
        parent_inline: true,
        type_map: &mut TypeDefs::default(),
    }))
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string.
/// Eg. `{ demo: string; };`
pub fn inline<T: Type>() -> String {
    datatype(&T::inline(
        DefOpts {
            parent_inline: true,
            type_map: &mut TypeDefs::default(),
        },
        &[],
    ))
}

/// Convert a DataType to a TypeScript string with an export.
/// Eg. `export type Foo = { demo: string; };`
pub fn export_datatype(def: &DataType) -> Result<String, String> {
    let inline_ts = datatype(def);

    let declaration = match &def {
        // Named struct
        DataType::Object(ObjectType {
            name,
            generics,
            fields,
            ..
        }) => {
            if name.is_empty() {
                return Err(
                    "Cannot export anonymous object. Try wrapping the type in a tuple struct which has the `ToDataType` derive macro on it.".to_string(),
                );
            }

            match fields.len() {
                0 => format!("type {name} = {inline_ts}"),
                _ => {
                    let generics = match generics.len() {
                        0 => "".into(),
                        _ => format!("<{}>", generics.to_vec().join(", ")),
                    };

                    format!("type {name}{generics} = {inline_ts}")
                }
            }
        }
        // Enum
        DataType::Enum(EnumType { name, generics, .. }) => {
            if name.is_empty() {
                return Err("Cannot export anonymous enum. Try wrapping the type in a tuple struct which has the `ToDataType` derive macro on it.".to_string());
            }

            let generics = match generics.len() {
                0 => "".into(),
                _ => format!("<{}>", generics.to_vec().join(", ")),
            };

            format!("type {name}{generics} = {inline_ts}")
        }
        // Unnamed struct
        DataType::Tuple(TupleType { name, generics, .. }) => {
            let generics = match generics.len() {
                0 => "".into(),
                _ => format!("<{}>", generics.to_vec().join(", ")),
            };

            format!("type {name}{generics} = {inline_ts}")
        }
        _ => return Err(format!("Type cannot be exported: {:?}", def)), // TODO: Can this be enforced at a type system level
    };

    Ok(format!("export {declaration}"))
}

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

/// Convert a DataType to a TypeScript string
/// Eg. `{ demo: string; }`
pub fn datatype(typ: &DataType) -> String {
    match &typ {
        DataType::Any => "any".into(),
        primitive_def!(i8 i16 i32 u8 u16 u32 f32 f64) => "number".into(),
        primitive_def!(usize isize i64 u64 i128 u128) => panic!(
            "TypeScript does not support integers larger than 54 bits. If you need to use these, we recommend serializing them as strings and using #[specta(type = String)] to override the type."
        ),
        primitive_def!(String char) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        DataType::Literal(literal) => literal.to_ts(),
        DataType::Nullable(def) => format!("{} | null", datatype(def)),
        DataType::Record(def) => {
            format!("Record<{}, {}>", datatype(&def.0), datatype(&def.1))
        }
        DataType::List(def) => format!("Array<{}>", datatype(def)),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "null".to_string(),
            [ty] => datatype(ty),
            tys => format!(
                "[{}]",
                tys.iter().map(datatype).collect::<Vec<_>>().join(", ")
            ),
        },
        DataType::Object(ObjectType {
            fields, tag, name, ..
        }) => match &fields[..] {
            [] => "null".to_string(),
            fields => {
                let mut field_sections = fields
                    .iter()
                    .filter(|f| f.flatten)
                    .map(|field| {
                        let type_str = datatype(&field.ty);
                        format!("({type_str})")
                    })
                    .collect::<Vec<_>>();

                let mut unflattened_fields = fields
                    .iter()
                    .filter(|f| !f.flatten)
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

                        format!("{key}: {}", datatype(ty))
                    })
                    .collect::<Vec<_>>();

                if let Some(tag) = tag {
                    unflattened_fields.push(format!("{tag}: \"{name}\""));
                }

                if !unflattened_fields.is_empty() {
                    field_sections.push(format!("{{ {} }}", unflattened_fields.join(", ")));
                }

                field_sections.join(" & ")
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
                            let typ = datatype(&DataType::Tuple(tuple.clone()));

                            format!("{{ {tag}: \"{sanitised_name}\" }} & {typ}")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                            let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                            fields.extend(
                                obj.fields
                                    .iter()
                                    .map(object_field_to_ts)
                                    .collect::<Vec<_>>(),
                            );

                            format!("{{ {} }}", fields.join(", "))
                        }
                        (EnumRepr::External, EnumVariant::Unit(_)) => {
                            format!("\"{sanitised_name}\"")
                        }
                        (EnumRepr::External, v) => {
                            let ts_values = datatype(&v.data_type());

                            format!("{{ {sanitised_name}: {ts_values} }}")
                        }
                        (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                        (EnumRepr::Untagged, v) => datatype(&v.data_type()),
                        (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                            format!("{{ {tag}: \"{sanitised_name}\" }}")
                        }
                        (EnumRepr::Adjacent { tag, content }, v) => {
                            let ts_values = datatype(&v.data_type());

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
                let generics = generics.iter().map(datatype).collect::<Vec<_>>().join(", ");

                format!("{name}<{generics}>")
            }
        },
        DataType::Generic(GenericType(ident)) => ident.to_string(),
    }
}

impl LiteralType {
    fn to_ts(&self) -> String {
        match self {
            Self::i8(v) => v.to_string(),
            Self::i16(v) => v.to_string(),
            Self::i32(v) => v.to_string(),
            Self::u8(v) => v.to_string(),
            Self::u16(v) => v.to_string(),
            Self::u32(v) => v.to_string(),
            Self::f32(v) => v.to_string(),
            Self::f64(v) => v.to_string(),
            Self::bool(v) => v.to_string(),
            Self::String(v) => format!(r#""{v}""#),
            Self::None => "null".to_string(),
        }
    }
}

/// convert an object field into a Typescript string
pub fn object_field_to_ts(field: &ObjectField) -> String {
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

    format!("{key}: {}", datatype(ty))
}

/// sanitise a string to be a valid Typescript key
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
