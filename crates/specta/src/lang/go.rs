use crate::*;

/// TODO
pub fn export<T: Type>() -> Result<String, String> {
    datatype(&T::definition(DefOpts {
        parent_inline: true,
        type_map: &mut TypeDefs::default(),
    }))
}

fn datatype(t: &DataTypeExt) -> Result<String, String> {
    todo!();
    // Ok(match t.inner {
    //     DataType::Any => "*interface{}".into(),
    //     primitive_def!(i8 u8 u16 i16 i32 isize usize) => "int".to_string(),
    //     primitive_def!(u32) => "uint32".to_string(),
    //     primitive_def!(i64) => "int64".to_string(),
    //     primitive_def!(u64) => "uint64".to_string(),
    //     primitive_def!(String) => "string".to_string(),
    //     primitive_def!(char) => todo!(), // I think this should be `byte` and not `rune` but i'm not certain.
    //     primitive_def!(bool) => "bool".to_string(),
    //     primitive_def!(f32) => "float32".to_string(),
    //     primitive_def!(f64) => "float64".to_string(),
    //     primitive_def!(i128 u128) => return Err("Go does not support 128 numbers!".to_owned()),
    //     DataType::List(t) => format!("[]{}", datatype(t)?),
    //     DataType::Tuple(_) => {
    //         // TODO: Add support for this.
    //         return Err(
    //             "Specta does not currently support exporting tuple types to Go.".to_owned(),
    //         );
    //     }
    //     DataType::Record(t) => format!("map[{}]{}", datatype(&t.0)?, datatype(&t.1)?),
    //     DataType::Generic(GenericType(t)) => t.to_string(),
    //     DataType::Reference { name, generics, .. } => match &generics[..] {
    //         [] => name.to_string(),
    //         generics => {
    //             let generics = generics
    //                 .iter()
    //                 .map(datatype)
    //                 .collect::<Result<Vec<_>, _>>()?
    //                 .join(", ");

    //             format!("{name}<{generics}>")
    //         }
    //     },
    //     DataType::Nullable(t) => format!("*{}", datatype(t)?),
    //     DataType::Object(ObjectType {
    //         name,
    //         generics,
    //         fields,
    //         tag,
    //         ..
    //     }) => {
    //         match &fields[..] {
    //             [] => "type {name} struct {}".to_string(),
    //             fields => {
    //                 let generics = (!generics.is_empty())
    //                     .then(|| format!("[{} any]", generics.join(", "))) // TODO: Replace the `any` interface with something more specific for the type?
    //                     .unwrap_or_default();

    //                 let fields = fields
    //                     .iter()
    //                     .map(|f| {
    //                         let name = &f.name;
    //                         let typ = datatype(&f.ty)?;
    //                         Ok(format!("\t{name} {typ}"))
    //                     })
    //                     .collect::<Result<Vec<_>, String>>()?
    //                     .join(", ");

    //                 let tag = tag
    //                     .clone()
    //                     .map(|t| format!("var {t}: string"))
    //                     .unwrap_or_default();

    //                 format!("type {name}{generics} struct {{ {fields}{tag} }}\n")
    //             }
    //         }
    //     }
    //     DataType::Literal(_) => return Err("Go does not support literal types!".to_owned()),
    //     _ => todo!(), // TODO: Remove from all exporters and replace with todo!() on variants
    // })
}
