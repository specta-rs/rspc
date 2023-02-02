use indoc::formatdoc;

use crate::*;

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
            PrimitiveType::String => "String",
            PrimitiveType::char => "Char",
            PrimitiveType::i8 => "Byte",
            PrimitiveType::i16 => "Short",
            PrimitiveType::isize | PrimitiveType::i32 => "Int",
            PrimitiveType::i64 => "Long",
            PrimitiveType::u8 => "UByte",
            PrimitiveType::u16 => "UShort",
            PrimitiveType::usize | PrimitiveType::u32 => "UInt",
            PrimitiveType::u64 => "ULong",
            PrimitiveType::bool => "Boolean",
            PrimitiveType::f32 => "Float",
            PrimitiveType::f64 => "Double",
            PrimitiveType::i128 | PrimitiveType::u128 => {
                return Err("Swift does not support 128 numbers!".to_owned())
            }
        }
        .to_string(),
        DataType::List(t) => format!("List<{}>", datatype(t)?),
        DataType::Tuple(_) => return Err("Kotlin does not support tuple types".to_owned()),
        DataType::Record(t) => format!("HashMap<{}, {}>", datatype(&t.0)?, datatype(&t.1)?),
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
            name,
            generics,
            fields,
            tag,
            ..
        }) => {
            let decl = match &fields[..] {
                [] => "class {name}".to_string(),
                fields => {
                    let generics = (!generics.is_empty())
                        .then(|| format!("<{}>", generics.join(", ")))
                        .unwrap_or_default();

                    let fields = fields
                        .iter()
                        .map(|f| {
                            let name = &f.name;
                            let typ = datatype(&f.ty)?;
                            let optional = matches!(f.ty, DataType::Nullable(_))
                                .then(|| "= null")
                                .unwrap_or_default();

                            Ok(format!("\tvar {name}: {typ}{optional}"))
                        })
                        .collect::<Result<Vec<_>, String>>()?
                        .join(", ");

                    let tag = tag
                        .clone()
                        .map(|t| format!("var {t}: String"))
                        .unwrap_or_default();

                    format!("data class {name}{generics} ({fields}{tag})")
                }
            };
            formatdoc! {
                r#"
                    @Serializable
                    {decl}\n
                "#
            }
        }
        DataType::Literal(_) => return Err("Kotlin does not support literal types!".to_owned()),
        _ => todo!(),
    })
}
