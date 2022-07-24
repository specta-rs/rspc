mod typedef;
mod typescript;

use std::{
    any::TypeId,
    cell::{Cell, RefCell},
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

pub use specta_macros::*;
pub use typedef::*;
pub use typescript::*;

/// TODO
pub type TypeDefs = HashMap<TypeId, Typedef>;

/// TODO
pub trait Type {
    fn def(defs: &mut TypeDefs) -> Typedef;
}

macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            fn def(_: &mut TypeDefs) -> Typedef {
                Typedef {
                    type_id: std::any::TypeId::of::<$i>(),
                    body: BodyDefinition::Primitive(PrimitiveType::$i),
                }
            }
        }
    )+};
}

#[macro_export]
macro_rules! upsert_def {
    ($defs:ident, $generic:ident) => {
        if let Some(def) = $defs.get(&TypeId::of::<$generic>()) {
            def.clone()
        } else {
            let def = <$generic as Type>::def($defs);
            $defs.insert(TypeId::of::<$generic>(), def.clone());
            def
        }
    };
    ($defs:ident) => {
        $crate::upsert_def!($defs, T)
    };
}

macro_rules! impl_tuple {
    (($($i:ident),*)) => {
        impl<$($i: Type + 'static),*> Type for ($($i),*) {
            fn def(defs: &mut TypeDefs) -> Typedef {
                Typedef {
                    type_id: std::any::TypeId::of::<($($i),*)>(),
                    body: BodyDefinition::Tuple {
                        name: None,
                        fields: vec![$($crate::upsert_def!(defs, $i)),*],
                    }
                }
            }
        }
    };
}

impl_primitives!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64
    bool char
    String
    Path
    PathBuf
);

impl_tuple!(());
// T = (T1)
impl_tuple!((T1, T2));
impl_tuple!((T1, T2, T3));
impl_tuple!((T1, T2, T3, T4));
impl_tuple!((T1, T2, T3, T4, T5));
impl_tuple!((T1, T2, T3, T4, T5, T6));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11));
impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12));

impl<'a> Type for &'a str {
    fn def(defs: &mut TypeDefs) -> Typedef {
        String::def(defs)
    }
}

impl<'a, T: Type + 'static> Type for &'a T {
    fn def(defs: &mut TypeDefs) -> Typedef {
        T::def(defs)
    }
}

impl<T: Type + 'static> Type for Vec<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        Typedef {
            type_id: std::any::TypeId::of::<Vec<T>>(),
            body: BodyDefinition::List(Box::new(upsert_def!(defs))),
        }
    }
}

impl<'a, T: Type + 'static> Type for &'a [T] {
    fn def(defs: &mut TypeDefs) -> Typedef {
        Typedef {
            type_id: std::any::TypeId::of::<&[T]>(),
            body: BodyDefinition::List(Box::new(upsert_def!(defs))),
        }
    }
}

impl<'a, const N: usize, T: Type + 'static> Type for [T; N] {
    fn def(defs: &mut TypeDefs) -> Typedef {
        upsert_def!(defs);

        Typedef {
            type_id: std::any::TypeId::of::<[T; N]>(),
            body: BodyDefinition::List(Box::new(upsert_def!(defs))),
        }
    }
}

impl<T: Type + 'static> Type for Option<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        Typedef {
            type_id: std::any::TypeId::of::<Option<T>>(),
            body: BodyDefinition::Nullable(Box::new(upsert_def!(defs))),
        }
    }
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type + 'static> Type for $container<T> {
            fn def(defs: &mut TypeDefs) -> Typedef {
                upsert_def!(defs)
            }
        }
    )+}
}

impl_containers!(Box Rc Arc Cell RefCell Mutex);

// TODO: UUID & chrono types
