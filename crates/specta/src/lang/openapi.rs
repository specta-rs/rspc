use openapiv3::{
    ArrayType, NumberType, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type,
};

use crate::*;

// pub fn to_openapi_export(def: &DataType) -> Result<openapiv3::Schema, String> {
//     Ok(match &def {
//         // Named struct
//         // DataType::Object(ObjectType {
//         //     name,
//         //     generics,
//         //     fields,
//         //     ..
//         // }) => match fields.len() {
//         //     0 => format!("export type {name} = {inline_ts}"),
//         //     _ => {
//         //         let generics = match generics.len() {
//         //             0 => "".into(),
//         //             _ => format!("<{}>", generics.to_vec().join(", ")),
//         //         };

//         //         format!("export interface {name}{generics} {inline_ts}")
//         //     }
//         // },
//         // // Enum
//         // DataType::Enum(EnumType { name, generics, .. }) => {
//         //     let generics = match generics.len() {
//         //         0 => "".into(),
//         //         _ => format!("<{}>", generics.to_vec().join(", ")),
//         //     };

//         //     format!("export type {name}{generics} = {inline_ts}")
//         // }
//         // // Unnamed struct
//         // DataType::Tuple(TupleType { name, .. }) => {
//         //     format!("export type {name} = {inline_ts}")
//         // }
//         _ => todo!(), // return Err(format!("Type cannot be exported: {:?}", def)),
//     })
// }

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

pub fn to_openapi(typ: &DataType) -> ReferenceOr<Schema> {
    let mut schema_data = SchemaData {
        nullable: false,
        deprecated: false, // TODO
        example: None,     // TODO
        title: None,       // TODO
        description: None, // TODO
        default: None,     // TODO
        ..Default::default()
    };

    match &typ {
        DataType::Any => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType::default())), // TODO: Use official "Any Type"
        }),

        primitive_def!(i8 i16 i32 isize u8 u16 u32 usize f32 f64) => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::Number(NumberType::default())), // TODO: Configure number type. Ts: `number`
        }),
        primitive_def!(i64 u64 i128 u128) => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::Number(NumberType::default())), // TODO: Configure number type. Ts: `bigint`
        }),
        primitive_def!(String char) => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::String(StringType::default())), // TODO: Configure string type. Ts: `string`
        }),
        primitive_def!(bool) => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::Boolean {}),
        }),
        // primitive_def!(Never) => "never".into(),
        DataType::Nullable(def) => {
            let schema = to_openapi(def);
            // schema.schema_data.nullable = true; // TODO
            schema
        }
        // DataType::Record(def) => {
        //     format!("Record<{}, {}>", to_openapi(&def.0), to_openapi(&def.1))
        // }
        DataType::List(def) => ReferenceOr::Item(Schema {
            schema_data,
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: Some(match to_openapi(def) {
                    ReferenceOr::Item(schema) => ReferenceOr::Item(Box::new(schema)),
                    ReferenceOr::Reference { reference } => ReferenceOr::Reference { reference },
                }),
                // TODO: This type is missing `Default`
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        }),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => {
                schema_data.nullable = true;
                ReferenceOr::Item(Schema {
                    schema_data,
                    schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType::default())), // TODO: This should be `null` type
                })
            }
            [ty] => to_openapi(ty),
            tys => todo!(),
        },
        DataType::Object(ObjectType {
            fields, tag, name, ..
        }) => match &fields[..] {
            [] => todo!(), // "null".to_string(),
            fields => {
                // let mut out = match tag {
                //     Some(tag) => vec![format!("{tag}: \"{name}\"")],
                //     None => vec![],
                // };

                // let field_defs = object_fields(fields);

                // out.extend(field_defs);

                // format!("{{ {} }}", out.join(", "))

                ReferenceOr::Item(Schema {
                    schema_data,
                    schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType {
                        properties: fields
                            .iter()
                            .map(
                                |ObjectField {
                                     name, ty, optional, ..
                                 }| {
                                    (
                                        name.clone(),
                                        match to_openapi(ty) {
                                            ReferenceOr::Item(v) => ReferenceOr::Item(Box::new(v)),
                                            ReferenceOr::Reference { reference } => {
                                                ReferenceOr::Reference { reference }
                                            }
                                        },
                                    )
                                },
                            )
                            .collect(),
                        ..Default::default()
                    })),
                })
            }
        },
        DataType::Enum(EnumType { variants, repr, .. }) => match &variants[..] {
            [] => todo!(), // "never".to_string(),
            variants => {
                // variants
                // .iter()
                // .map(|variant| {
                //     let sanitised_name = sanitise_name(variant.name());

                //     match (repr, variant) {
                //         (EnumRepr::Internal { tag }, EnumVariant::Unit(_)) => {
                //             format!("{{ {tag}: \"{sanitised_name}\" }}")
                //         }
                //         (EnumRepr::Internal { tag }, EnumVariant::Unnamed(tuple)) => {
                //             let typ = to_openapi(&DataType::Tuple(tuple.clone()));

                //             format!("{{ {tag}: \"{sanitised_name}\" }} & {typ}")
                //         }
                //         (EnumRepr::Internal { tag }, EnumVariant::Named(obj)) => {
                //             let mut fields = vec![format!("{tag}: \"{sanitised_name}\"")];

                //             fields.extend(object_fields(&obj.fields));

                //             format!("{{ {} }}", fields.join(", "))
                //         }
                //         (EnumRepr::External, EnumVariant::Unit(_)) => {
                //             format!("\"{sanitised_name}\"")
                //         }
                //         (EnumRepr::External, v) => {
                //             let ts_values = to_openapi(&v.data_type());

                //             format!("{{ {sanitised_name}: {ts_values} }}")
                //         }
                //         (EnumRepr::Untagged, EnumVariant::Unit(_)) => "null".to_string(),
                //         (EnumRepr::Untagged, v) => to_openapi(&v.data_type()),
                //         (EnumRepr::Adjacent { tag, .. }, EnumVariant::Unit(_)) => {
                //             format!("{{ {tag}: \"{sanitised_name}\" }}")
                //         }
                //         (EnumRepr::Adjacent { tag, content }, v) => {
                //             let ts_values = to_openapi(&v.data_type());

                //             format!("{{ {tag}: \"{sanitised_name}\", {content}: {ts_values} }}")
                //         }
                //     }
                // })
                // .collect::<Vec<_>>()
                // .join(" | ");

                ReferenceOr::Item(Schema {
                    schema_data,
                    schema_kind: SchemaKind::AnyOf {
                        any_of: variants
                            .iter()
                            .map(|variant| match variant {
                                EnumVariant::Unit(_) => ReferenceOr::Item(Schema {
                                    schema_data: Default::default(),
                                    schema_kind: SchemaKind::Type(Type::Object(
                                        openapiv3::ObjectType::default(), // TODO: Is this correct?
                                    )),
                                }),
                                EnumVariant::Unnamed(tuple) => {
                                    to_openapi(&DataType::Tuple(tuple.clone()))
                                }
                                EnumVariant::Named(obj) => {
                                    to_openapi(&DataType::Object(obj.clone()))
                                }
                            })
                            .collect(),
                    },
                })
            }
        },
        DataType::Reference { name, generics, .. } => match &generics[..] {
            [] => ReferenceOr::Item(Schema {
                schema_data,
                schema_kind: SchemaKind::OneOf {
                    one_of: vec![ReferenceOr::Reference {
                        reference: format!("#/components/schemas/{}", name),
                    }],
                },
            }),
            generics => {
                // let generics = generics
                //     .iter()
                //     .map(to_openapi)
                //     .collect::<Vec<_>>()
                //     .join(", ");

                // format!("{name}<{generics}>")
                todo!();
            }
        },
        // DataType::Generic(ident) => ident.to_string(),
        x => {
            println!("{:?} {:?}", x, typ);
            todo!();
        }
    }
}
