use thiserror::Error;

use crate::*;

/// Allows you to configure how Specta's Typescript exporter will deal with BigInt types (i64 u64 i128 u128).
#[derive(Default)]
pub enum BigIntExportBehavior {
    /// Export BigInt as a Typescript `string`
    /// WARNING: Specta takes no responsibility that the Rust number is encoded as a string.
    /// Make sure you instruct serde <https://github.com/serde-rs/json/issues/329#issuecomment-305608405> or your other serializer of this.
    String,
    /// Export BigInt as a Typescript `number`.
    /// WARNING: `JSON.parse` in JS will truncate your number resulting in data loss so ensure your deserializer supports bigint types.
    Number,
    /// Export BigInt as a Typescript `BigInt`.
    /// WARNING: Specta takes no responsibility that the Rust number is decoded into this type on the frontend.
    /// Ensure you deserializer is able to do this.
    BigInt,
    /// Abort the export with an error
    /// This is the default behavior because without integration from your serializer and deserializer we can't guarantee data loss won't occur.
    #[default]
    Fail,
    /// Same as `Self::Fail` but it allows a library to configure the message shown to the end user.
    #[doc(hidden)]
    FailWithReason(&'static str),
}

/// Allows you to configure how Specta's Typescript exporter will deal with Rust doc comments.
#[derive(Default)]
pub enum CommentStyle {
    /// Export them in a JSDoc style
    #[default]
    JsDoc,
    /// Do not export them
    Disabled,
}

/// TODO
impl CommentStyle {
    /// TODO
    pub fn render(&self, comments: &'static [&'static str]) -> String {
        match self {
            Self::JsDoc => {
                if comments.len() == 0 {
                    return "".to_owned();
                }

                let mut result = "/**\n".to_owned();
                for comment in comments {
                    result.push_str(&format!(" * {}\n", comment));
                }
                result.push_str(" */\n");
                result
            }
            Self::Disabled => "".to_owned(),
        }
    }
}

/// allows you to control the behavior of the Typescript exporter
#[derive(Default)]
pub struct ExportConfiguration {
    /// control the bigint exporting behavior
    pub bigint: BigIntExportBehavior,
    /// control the style of exported comments
    pub comment_style: CommentStyle,
}

#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum TsExportError {
    #[error("Failed to export type '{}' on field `{}`: {err}", .ty_name.unwrap_or_default(), .field_name.clone().unwrap_or_default())]
    WithCtx {
        // TODO: Handle this better. Make `ty_name` non optional
        ty_name: Option<&'static str>,
        field_name: Option<&'static str>,
        err: Box<TsExportError>,
    },
    #[error("Your Specta configuration forbids exporting BigInt types (i64, u64, i128, u128) because we don't know if your se/deserializer supports it. You can change this behavior by editing your `ExportConfiguration`")]
    BigIntForbidden,
    #[error("Cannot export anonymous object. Try wrapping the type in a tuple struct which has the `ToDataType` derive macro on it.")]
    AnonymousObject,
    #[error("Cannot export anonymous enum. Try wrapping the type in a tuple struct which has the `ToDataType` derive macro on it.")]
    AnonymousEnum,
    #[error("Type cannot be exported: {0:?}")]
    CannotExport(DataTypeExt),
    #[error("Cannot export type due to an internal error. This likely is a bug in Specta itself and not your code: {0}")]
    InternalError(&'static str),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string with an export.
/// Eg. `export type Foo = { demo: string; };`
pub fn export<T: Type>(conf: &ExportConfiguration) -> Result<String, TsExportError> {
    export_datatype(
        conf,
        &T::definition(DefOpts {
            parent_inline: true,
            type_map: &mut TypeDefs::default(),
        }),
    )
}

/// Convert a type which implements [`Type`](crate::Type) to a TypeScript string.
/// Eg. `{ demo: string; };`
pub fn inline<T: Type>(conf: &ExportConfiguration) -> Result<String, TsExportError> {
    datatype(
        conf,
        &T::inline(
            DefOpts {
                parent_inline: true,
                type_map: &mut TypeDefs::default(),
            },
            &[],
        ),
    )
}

/// Convert a DataType to a TypeScript string with an export.
/// Eg. `export type Foo = { demo: string; };`
///
// TODO: Accept `DataTypeExt` or `DataType`. This is hard because we take it by reference
pub fn export_datatype(
    conf: &ExportConfiguration,
    def: &DataTypeExt,
) -> Result<String, TsExportError> {
    let inline_ts = datatype(conf, &def.inner).map_err(|err| TsExportError::WithCtx {
        ty_name: Some(def.name),
        field_name: None,
        err: Box::new(err),
    })?;

    let declaration = match &def.inner {
        // Named struct
        DataType::Object(ObjectType {
            name,
            generics,
            fields,
            ..
        }) => {
            if name.is_empty() {
                return Err(TsExportError::AnonymousObject);
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
                return Err(TsExportError::AnonymousEnum);
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
        _ => return Err(TsExportError::CannotExport(def.clone())), // TODO: Can this be enforced at a type system level
    };

    let comments = conf.comment_style.render(def.comments);
    Ok(format!("{comments}export {declaration}"))
}

/// Convert a DataType to a TypeScript string
/// Eg. `{ demo: string; }`
pub fn datatype(conf: &ExportConfiguration, typ: &DataType) -> Result<String, TsExportError> {
    Ok(match &typ {
        DataType::Any => "any".into(),
        primitive_def!(i8 i16 i32 u8 u16 u32 f32 f64) => "number".into(),
        primitive_def!(usize isize i64 u64 i128 u128) => match conf.bigint {
            BigIntExportBehavior::String => "string".into(),
            BigIntExportBehavior::Number => "number".into(),
            BigIntExportBehavior::BigInt => "BigInt".into(),
            BigIntExportBehavior::Fail => return Err(TsExportError::BigIntForbidden),
            BigIntExportBehavior::FailWithReason(reason) => {
                return Err(TsExportError::Other(reason.to_owned()))
            }
        },
        primitive_def!(String char) => "string".into(),
        primitive_def!(bool) => "boolean".into(),
        DataType::Literal(literal) => literal.to_ts(),
        DataType::Nullable(def) => format!("{} | null", datatype(conf, def)?),
        DataType::Record(def) => {
            format!(
                // We use this isn't of `Record<K, V>` to avoid issues with circular references.
                "{{ [key: {}]: {} }}",
                datatype(conf, &def.0)?,
                datatype(conf, &def.1)?
            )
        }
        // We use `T[]` instead of `Array<T>` to avoid issues with circular references.
        DataType::List(def) => format!("{}[]", datatype(conf, def)?),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "null".to_string(),
            [ty] => datatype(conf, ty)?,
            tys => format!(
                "[{}]",
                tys.iter()
                    .map(|v| datatype(conf, v))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
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
                        datatype(conf, &field.ty)
                            .map(|type_str| format!("({type_str})"))
                            .map_err(|err| TsExportError::WithCtx {
                                ty_name: None,
                                field_name: Some(field.name.clone()),
                                err: Box::new(err),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

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

                        datatype(conf, ty)
                            .map(|v| format!("{key}: {v}"))
                            .map_err(|err| TsExportError::WithCtx {
                                ty_name: None,
                                field_name: Some(field.name),
                                err: Box::new(err),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                if let Some(tag) = tag {
                    unflattened_fields.push(format!("{tag}: \"{name}\""));
                }

                if !unflattened_fields.is_empty() {
                    field_sections.push(format!("{{ {} }}", unflattened_fields.join("; ")));
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

                    Ok(match (repr, variant) {
                        (EnumRepr::Internal { tag }, EnumVariant::Unit(_)) => {
                            format!("{{ {tag}: \"{sanitised_name}\" }}")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                            let typ =
                                datatype(conf, &DataType::Tuple(tuple.clone())).map_err(|err| {
                                    TsExportError::WithCtx {
                                        ty_name: None,
                                        field_name: Some(variant.name()),
                                        err: Box::new(err),
                                    }
                                })?;

                            format!("({{ {tag}: \"{sanitised_name}\" }} & {typ})")
                        }
                        (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                            let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                            fields.extend(
                                obj.fields
                                    .iter()
                                    .map(|v| object_field_to_ts(conf, v))
                                    .collect::<Result<Vec<_>, _>>()?,
                            );

                            format!("{{ {} }}", fields.join("; "))
                        }
                        (EnumRepr::External, EnumVariant::Unit(_)) => {
                            format!("\"{sanitised_name}\"")
                        }
                        (EnumRepr::External, v) => {
                            let ts_values = datatype(conf, &v.data_type()).map_err(|err| {
                                TsExportError::WithCtx {
                                    ty_name: None,
                                    field_name: Some(variant.name()),
                                    err: Box::new(err),
                                }
                            })?;

                            format!("{{ {sanitised_name}: {ts_values} }}")
                        }
                        (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                        (EnumRepr::Untagged, v) => {
                            datatype(conf, &v.data_type()).map_err(|err| {
                                TsExportError::WithCtx {
                                    ty_name: None,
                                    field_name: Some(variant.name()),
                                    err: Box::new(err),
                                }
                            })?
                        }
                        (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                            format!("{{ {tag}: \"{sanitised_name}\" }}")
                        }
                        (EnumRepr::Adjacent { tag, content }, v) => {
                            let ts_values = datatype(conf, &v.data_type()).map_err(|err| {
                                TsExportError::WithCtx {
                                    ty_name: None,
                                    field_name: Some(variant.name()),
                                    err: Box::new(err),
                                }
                            })?;

                            format!("{{ {tag}: \"{sanitised_name}\"; {content}: {ts_values} }}")
                        }
                    })
                })
                .collect::<Result<Vec<_>, TsExportError>>()?
                .join(" | "),
        },
        DataType::Reference { name, generics, .. } => match &generics[..] {
            [] => name.to_string(),
            generics => {
                let generics = generics
                    .iter()
                    .map(|v| datatype(conf, v))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                format!("{name}<{generics}>")
            }
        },
        DataType::Generic(GenericType(ident)) => ident.to_string(),
        DataType::Placeholder => {
            return Err(TsExportError::InternalError(
                "Attempted to export a placeholder!",
            ))
        }
    })
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
pub fn object_field_to_ts(
    conf: &ExportConfiguration,
    field: &ObjectField,
) -> Result<String, TsExportError> {
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

    Ok(format!("{key}: {}", datatype(conf, ty)?))
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
