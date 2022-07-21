use std::{
    any::{type_name, TypeId},
    collections::HashSet,
    marker::PhantomData,
    path::Path,
};

use ts_rs::{Dependency, ExportError, TS};

pub struct ExtendTypeDef<T: TS, TT: TS> {
    phantom: PhantomData<(T, TT)>,
}

impl<T: TS, TT: TS> ExtendTypeDef<T, TT> {}

impl<T: TS, TT: TS> TS for ExtendTypeDef<T, TT> {
    fn name() -> String {
        format!("ExtendTypeDef<{}, {}>", T::name(), TT::name())
    }

    fn name_with_type_args(args: Vec<String>) -> String {
        assert_eq!(
            args.len(),
            2,
            "called ExtendTypeDef::name_with_type_args with {} args",
            args.len()
        );
        format!("{} | {}", args[0], args[1])
    }

    fn inline() -> String {
        if TypeId::of::<T>() == TypeId::of::<TT>() || TypeId::of::<TT>() == TypeId::of::<()>() {
            format!("{}", T::inline())
        } else if TypeId::of::<T>() == TypeId::of::<()>() {
            format!("{}", TT::name())
        } else {
            format!("{} & {}", T::inline(), TT::name()) // TODO: This line means `TT` must not be generic. That limitation should be fixed!
        }
    }

    fn dependencies(deps: &mut std::collections::HashMap<std::any::TypeId, Dependency>) {
        if !deps.contains_key(&TypeId::of::<T>()) {
            if let Some(dep) = Dependency::from_ty::<T>() {
                deps.insert(dep.type_id, dep);
            }

            T::dependencies(deps);
        }

        if !deps.contains_key(&TypeId::of::<TT>()) {
            if let Some(dep) = Dependency::from_ty::<TT>() {
                deps.insert(dep.type_id, dep);
            }

            TT::dependencies(deps);
        }
    }

    fn transparent() -> bool {
        true
    }

    fn export_to(
        path: impl AsRef<Path>,
        exported_types: &mut HashSet<TypeId>,
    ) -> Result<(), ExportError> {
        if !exported_types.contains(&TypeId::of::<T>()) {
            exported_types.insert(std::any::TypeId::of::<T>());
            let _ = T::export_to(path.as_ref(), exported_types); // TODO: Error handling
        }

        if !exported_types.contains(&TypeId::of::<TT>()) {
            exported_types.insert(std::any::TypeId::of::<TT>());
            let _ = TT::export_to(path, exported_types); // TODO: Error handling
        }

        Ok(())
    }
}
