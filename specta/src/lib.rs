mod datatype;
mod r#enum;
mod object;
mod tuple;
mod typescript;

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

pub use datatype::*;
pub use object::*;
pub use r#enum::*;
pub use specta_macros::*;
pub use tuple::*;
pub use typescript::*;

pub type TypeDefs = HashMap<String, DataType>;

pub trait Type {
    fn def(defs: &mut TypeDefs) -> DataType;
    fn base(defs: &mut TypeDefs) -> DataType;
    fn name() -> Option<String>;
    fn refr() -> DataType;
}

macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            fn def(_: &mut TypeDefs) -> DataType {
                DataType::Primitive(PrimitiveType::$i)
            }

            fn base(defs: &mut TypeDefs) -> DataType {
                Self::def(defs)
            }

            fn name() -> Option<String> {
                None
            }

            fn refr() -> DataType {
                unreachable!()
            }
        }
    )+};
}

#[macro_export]
macro_rules! upsert_def {
    ($defs:ident, $generic:ident) => {
        if let Some(name) = <$generic as Type>::name() {
            if let Some(def) = $defs.get(&name) {
                def.clone()
            } else {
                let def = <$generic as Type>::def($defs);
                $defs.insert(name, def.clone());
                def
            }
        } else {
            <$generic as Type>::def($defs)
        }
    };
    ($defs:ident) => {
        $crate::upsert_def!($defs, T)
    };
}

macro_rules! impl_tuple {
    (($($i:ident),*)) => {
        impl<$($i: Type + 'static),*> Type for ($($i),*) {
            fn def(_defs: &mut TypeDefs) -> DataType {
                DataType::Tuple(TupleType {
                    name: stringify!(($($i),*)).to_string(),
                    id: std::any::TypeId::of::<($($i),*)>(),
                    fields: vec![$($crate::upsert_def!(_defs, $i)),*],
                })
            }

            fn base(defs: &mut TypeDefs) -> DataType {
                Self::def(defs)
            }

            fn name() -> Option<String> {
                None
            }

            fn refr() -> DataType {
                unreachable!()
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
    fn def(defs: &mut TypeDefs) -> DataType {
        String::def(defs)
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        String::name()
    }

    fn refr() -> DataType {
        String::refr()
    }
}

impl<'a, T: Type + 'static> Type for &'a T {
    fn def(defs: &mut TypeDefs) -> DataType {
        T::def(defs)
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        T::name()
    }

    fn refr() -> DataType {
        T::refr()
    }
}

impl<T: Type + 'static> Type for Vec<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        T::base(defs)
    }

    fn name() -> Option<String> {
        T::name()
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

impl<'a, T: Type + 'static> Type for &'a [T] {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        T::base(defs)
    }

    fn name() -> Option<String> {
        T::name()
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

impl<'a, const N: usize, T: Type + 'static> Type for [T; N] {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        T::base(defs)
    }

    fn name() -> Option<String> {
        T::name()
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

impl<T: Type + 'static> Type for Option<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::Nullable(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        T::base(defs)
    }

    fn name() -> Option<String> {
        T::name()
    }

    fn refr() -> DataType {
        DataType::Nullable(Box::new(T::refr()))
    }
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type + 'static> Type for $container<T> {
            fn def(defs: &mut TypeDefs) -> DataType {
                upsert_def!(defs)
            }

            fn base(defs: &mut TypeDefs) -> DataType {
                T::base(defs)
            }

            fn name() -> Option<String> {
                T::name()
            }

            fn refr() -> DataType {
                T::refr()
            }
        }
    )+}
}

impl_containers!(Box Rc Arc Cell RefCell Mutex);

impl<T: Type> Type for HashSet<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

impl<T: Type> Type for BTreeSet<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

impl<K: Type, V: Type> Type for HashMap<K, V> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::Record(Box::new((K::def(defs), V::def(defs))))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::Record(Box::new((K::refr(), V::refr())))
    }
}

impl<K: Type, V: Type> Type for BTreeMap<K, V> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::Record(Box::new((K::def(defs), V::def(defs))))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::Record(Box::new((K::refr(), V::refr())))
    }
}

#[cfg(feature = "indexmap")]
impl<T: Type> Type for indexmap::IndexSet<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::List(Box::new(upsert_def!(defs)))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::List(Box::new(T::refr()))
    }
}

#[cfg(feature = "indexmap")]
impl<K: Type, V: Type> Type for indexmap::IndexMap<K, V> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::Record(Box::new((K::def(defs), V::def(defs))))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::Record(Box::new((K::refr(), V::refr())))
    }
}

#[cfg(feature = "serde")]
impl<K: Type, V: Type> Type for serde_json::Map<K, V> {
    fn def(defs: &mut TypeDefs) -> DataType {
        DataType::Record(Box::new((K::def(defs), V::def(defs))))
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::Record(Box::new((K::refr(), V::refr())))
    }
}

#[cfg(feature = "serde")]
impl Type for serde_json::Value {
    fn def(_defs: &mut TypeDefs) -> DataType {
        DataType::Any
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        None
    }

    fn refr() -> DataType {
        DataType::Any
    }
}

#[cfg(feature = "uuid")]
impl Type for uuid::Uuid {
    fn def(defs: &mut TypeDefs) -> DataType {
        String::def(defs)
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        String::name()
    }

    fn refr() -> DataType {
        String::refr()
    }
}

#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> Type for chrono::DateTime<T> {
    fn def(defs: &mut TypeDefs) -> DataType {
        String::def(defs)
    }

    fn base(defs: &mut TypeDefs) -> DataType {
        Self::def(defs)
    }

    fn name() -> Option<String> {
        String::name()
    }

    fn refr() -> DataType {
        String::refr()
    }
}
