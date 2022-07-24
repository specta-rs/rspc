mod typedef;
mod typescript;

use std::{
    any::TypeId,
    cell::{Cell, RefCell},
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
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
            fn def(defs: &mut TypeDefs) -> Typedef {
                Typedef {
                    name: stringify!($i).into(),
                    type_id: std::any::TypeId::of::<$i>(),
                    body: BodyDefinition::Primitive(PrimitiveType::$i),
                }
            }
        }
    )+};
}

macro_rules! impl_tuple {
    (()) => {
        impl Type for () {
            fn def(defs: &mut TypeDefs) -> Typedef {
                Typedef {
                    name: stringify!(()).into(),
                    type_id: std::any::TypeId::of::<()>(),
                    body: BodyDefinition::Tuple(vec![]),
                }
            }
        }
    };
    (($($i:ident),+)) => {
        impl<$($i: Type + 'static),+> Type for ($($i),+) {
            fn def(defs: &mut TypeDefs) -> Typedef {
                Typedef {
                    name: format!(
                        "({})",
                        vec![$(
                            if let Some(def) = defs.get(&TypeId::of::<$i>()) {
                                def.clone()
                            } else {
                                let def = <$i as Type>::def(defs);
                                defs.insert(TypeId::of::<$i>(), def.clone());
                                def
                            }.name
                        ),*].join(", ")
                    ),
                    type_id: std::any::TypeId::of::<($($i),+)>(),
                    body: BodyDefinition::Tuple(vec![$(
                        if let Some(def) = defs.get(&TypeId::of::<$i>()) {
                            def.clone()
                        } else {
                            let def = <$i as Type>::def(defs);
                            defs.insert(TypeId::of::<$i>(), def.clone());
                            def
                        }
                    ),+]),
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
        let def = if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        };

        Typedef {
            name: format!("Vec<{}>", def.name),
            type_id: std::any::TypeId::of::<Vec<T>>(),
            body: BodyDefinition::List(Box::new(def)),
        }
    }
}

impl<'a, T: Type + 'static> Type for &'a [T] {
    fn def(defs: &mut TypeDefs) -> Typedef {
        let def = if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        };

        Typedef {
            name: format!("[{}]", def.name),
            type_id: std::any::TypeId::of::<&[T]>(),
            body: BodyDefinition::List(Box::new(def)),
        }
    }
}

impl<'a, const N: usize, T: Type + 'static> Type for [T; N] {
    fn def(defs: &mut TypeDefs) -> Typedef {
        let def = if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        };

        Typedef {
            name: format!("[{}; {}]", def.name, N),
            type_id: std::any::TypeId::of::<[T; N]>(),
            body: BodyDefinition::List(Box::new(def)),
        }
    }
}

impl<T: Type + 'static> Type for Option<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        let def = if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        };

        Typedef {
            name: format!("Option<{}>", def.name),
            type_id: std::any::TypeId::of::<Option<T>>(),
            body: BodyDefinition::Nullable(Box::new(def)),
        }
    }
}

impl<T: Type + 'static> Type for Box<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        }
    }
}

impl<T: Type + 'static> Type for RefCell<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        }
    }
}

impl<T: Type + 'static> Type for Arc<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        }
    }
}

impl<T: Type + 'static> Type for Rc<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        }
    }
}

impl<T: Type + 'static> Type for Cell<T> {
    fn def(defs: &mut TypeDefs) -> Typedef {
        if let Some(def) = defs.get(&TypeId::of::<T>()) {
            def.clone()
        } else {
            let def = <T as Type>::def(defs);
            defs.insert(TypeId::of::<T>(), def.clone());
            def
        }
    }
}

// TODO: UUID & chrono types
