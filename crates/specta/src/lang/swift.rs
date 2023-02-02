use crate::*;
use indoc::*;

/// TODO
pub fn export<T: Type>() -> Result<String, String> {
    datatype(&T::definition(DefOpts {
        parent_inline: true,
        type_map: &mut TypeDefs::default(),
    }))
}

fn datatype(t: &DataTypeExt) -> Result<String, String> {
    Ok(match t.inner {
        DataType::Primitive(p) => match p {
            PrimitiveType::String | PrimitiveType::char => "String",
            PrimitiveType::i8 => "Int8",
            PrimitiveType::u8 => "UInt8",
            PrimitiveType::i16 => "Int16",
            PrimitiveType::u16 => "UInt16",
            PrimitiveType::usize => "UInt",
            PrimitiveType::isize => "Int",
            PrimitiveType::i32 => "Int32",
            PrimitiveType::u32 => "UInt32",
            PrimitiveType::i64 => "Int64",
            PrimitiveType::u64 => "UInt64",
            PrimitiveType::bool => "Bool",
            PrimitiveType::f32 => "Float",
            PrimitiveType::f64 => "Double",
            PrimitiveType::i128 | PrimitiveType::u128 => {
                return Err("Swift does not support 128 numbers!".to_owned());
            }
        }
        .to_string(),
        DataType::Any => "Codable".to_string(),
        DataType::List(t) => format!("[{}]", datatype(&t)?),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "CodableVoid".to_string(),
            [ty] => datatype(ty)?,
            tys => format!(
                "({})",
                tys.iter()
                    .map(datatype)
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ),
        },
        DataType::Record(t) => format!("[{}: {}]", datatype(&t.0)?, datatype(&t.1)?),
        DataType::Generic(GenericType(t)) => t.to_string(),
        DataType::Reference { name, generics, .. } => match &generics[..] {
            [] => name.to_string(),
            generics => {
                let generics = generics
                    .iter()
                    .map(datatype)
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");

                format!("{name}<{generics}>")
            }
        },
        DataType::Nullable(t) => format!("{}?", datatype(t)?),
        DataType::Object(ObjectType {
            fields,
            tag,
            name,
            generics,
            ..
        }) => match &fields[..] {
            [] => "CodableVoid".to_string(),
            fields => {
                // TODO: Handle invalid field names
                let generics = (!generics.is_empty())
                    .then(|| {
                        format!(
                            "<{}>",
                            generics
                                .iter()
                                .map(|g| format!("{}: Codable", g))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })
                    .unwrap_or_default();

                let fields = fields
                    .iter()
                    .map(|f| {
                        let name = &f.name;
                        let typ = datatype(&f.ty)?;

                        Ok(format!("\tpublic let {name}: {typ}"))
                    })
                    .collect::<Result<Vec<_>, String>>()?
                    .join("\n");

                let tag = tag
                    .clone()
                    .map(|t| format!("\t{t}: String"))
                    .unwrap_or_default();

                formatdoc! {
                    r#"
                        public struct {name}{generics}: Codable {{
                        {tag}{fields}
                        }}
                    "#
                }
            }
        },
        DataType::Literal(_) => return Err("Swift does not support literal types!".to_owned()),
        _ => todo!(),
    })
}
