//! This file is the magic that allows switching Specta versions.

#[cfg(not(feature = "specta2"))]
mod a {
    pub use specta::Type;

    use specta::{DefOpts, TypeDefs};

    use crate::internal::ProcedureDataType;

    #[allow(clippy::unwrap_used)] // TODO
    pub fn typedef<TArg: Type, TResult: Type>(defs: &mut TypeDefs) -> ProcedureDataType {
        ProcedureDataType {
            arg_ty: <TArg as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            )
            .unwrap(),
            result_ty: <TResult as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            )
            .unwrap(),
        }
    }
}
#[cfg(not(feature = "specta2"))]
pub use a::*;

#[cfg(feature = "specta2")]
mod b {
    pub use specta2::Type;
    use specta2::{
        internal::interop::{specta_v2_to_v1, specta_v2_type_map_to_v1_type_defs},
        TypeMap,
    };

    use crate::internal::ProcedureDataType;

    #[allow(clippy::unwrap_used)] // TODO
    pub fn typedef<TArg: Type, TResult: Type>(defs: &mut specta::TypeDefs) -> ProcedureDataType {
        let mut type_map = TypeMap::default();
        let arg_ty = specta_v2_to_v1(TArg::reference(&mut type_map, &[]).inner);
        let result_ty = specta_v2_to_v1(TResult::reference(&mut type_map, &[]).inner);

        specta_v2_type_map_to_v1_type_defs(type_map, defs);

        ProcedureDataType { arg_ty, result_ty }
    }
}

#[cfg(feature = "specta2")]
pub use b::*;
