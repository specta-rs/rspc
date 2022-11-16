use crate::{
    DataType, DefOpts, ObjectField, ObjectType, ToDataType, TypeDefs, TypedCommandArg,
    TypedCommandResult,
};

#[derive(Debug, ToDataType)]
#[specta(crate = "crate")]
pub struct CommandDataType {
    pub name: &'static str,
    pub input: Option<DataType>,
    pub result: DataType,
}

pub trait TypedCommand<TMarker> {
    fn to_datatype(
        name: &'static str,
        type_map: &mut TypeDefs,
        fields: &[&'static str],
    ) -> CommandDataType;
}

impl<TResultMarker, TResult: TypedCommandResult<TResultMarker>> TypedCommand<TResultMarker>
    for fn() -> TResult
{
    fn to_datatype(
        name: &'static str,
        type_map: &mut TypeDefs,
        _fields: &[&'static str],
    ) -> CommandDataType {
        CommandDataType {
            name,
            input: None,
            result: TResult::to_datatype(DefOpts {
                parent_inline: false,
                type_map,
            }),
        }
    }
}

macro_rules! impl_typed_command {
    ( impl $($i:ident),* ) => {
       paste::paste! {
            impl<
                    TResultMarker,
                    TResult: TypedCommandResult<TResultMarker>,
                    $([<$i Marker>]),*,
                    $($i: TypedCommandArg<[<$i Marker>]>),*
                > TypedCommand<(TResultMarker, $([<$i Marker>]),*)> for fn($($i),*) -> TResult
            {
                fn to_datatype(
                    name: &'static str,
                    type_map: &mut TypeDefs,
                    fields: &[&'static str],
                ) -> CommandDataType {
                    let mut fields = fields.into_iter();
                    CommandDataType {
                        name,
                        input: Some(DataType::Object(ObjectType {
                            name: "_unreachable_".into(),
                            generics: vec![],
                            fields: [
                                $(
                                    $i::to_datatype(DefOpts {
                                        parent_inline: false,
                                        type_map,
                                    })
                                    .map(|ty| ObjectField {
                                        name: fields.next().expect("Tauri Specta reached an unreachable state. The macro returns the incorrect number of fields. Please file this as a bug on GitHub!").to_string(),
                                        ty: ty,
                                        optional: false,
                                        flatten: false,
                                    })
                                ),*,
                            ]
                            .into_iter()
                            .filter_map(|v| v)
                            .collect(),
                            tag: None,
                            type_id: None,
                        })),
                        result: TResult::to_datatype(DefOpts {
                            parent_inline: false,
                            type_map,
                        }),
                    }
                }
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_typed_command!(impl $i2 $(, $i)* );
        impl_typed_command!($($i),*);
    };
    () => {};
}

impl_typed_command!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

pub fn export_command_datatype<TMarker, T: TypedCommand<TMarker>>(
    _: T,
    name: &'static str,
    type_map: &mut TypeDefs,
    fields: &[&'static str],
) -> CommandDataType {
    T::to_datatype(name, type_map, fields)
}
