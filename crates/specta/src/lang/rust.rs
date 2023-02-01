use crate::*;

/// TODO
pub fn export<T: Type>() -> Result<String, String> {
    datatype(&T::definition(DefOpts {
        parent_inline: true,
        type_map: &mut TypeDefs::default(),
    }))
}

fn datatype(t: &DataTypeExt) -> Result<String, String> {
    // TODO: This system does lossy type conversions. That is something I want to fix in the future but for now this works. Eg. `HashSet<T>` will be exported as `Vec<T>`
    // TODO: Serde serialize + deserialize on types

    Ok(match t.inner {
        DataType::Any => "serde_json::Value".to_owned(),
        DataType::Primitive(ty) => ty.to_rust_str().to_owned(),
        DataType::Literal(_) => todo!(),
        DataType::Nullable(t) => format!("Option<{}>", datatype(t)?),
        DataType::Record(t) => format!("HashMap<{}, {}>", datatype(&t.0)?, datatype(&t.1)?),
        DataType::List(t) => format!("Vec<{}>", datatype(t)?),
        DataType::Tuple(TupleType { fields, .. }) => match &fields[..] {
            [] => "()".to_string(),
            [ty] => datatype(ty)?,
            tys => format!(
                "({})",
                tys.iter()
                    .map(|v| datatype(v))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ),
        },
        DataType::Object(ObjectType {
            name,
            generics,
            fields,
            tag,
            ..
        }) => match &fields[..] {
            [] => "struct {name}".to_string(),
            fields => {
                let generics = (!generics.is_empty())
                    .then(|| format!("<{}>", generics.join(", ")))
                    .unwrap_or_default();

                let fields = fields
                    .iter()
                    .map(|f| {
                        let name = &f.name;
                        let typ = datatype(&f.ty)?;
                        Ok(format!("\t{name}: {typ}"))
                    })
                    .collect::<Result<Vec<_>, String>>()?
                    .join(", ");

                let tag = tag
                    .clone()
                    .map(|t| format!("{t}: String"))
                    .unwrap_or_default();

                format!("struct {name}{generics} {{ {fields}{tag} }}\n")
            }
        },
        DataType::Enum(_) => todo!(),
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
        DataType::Generic(GenericType(t)) => t.to_string(),
    })
}
