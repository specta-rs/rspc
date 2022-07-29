mod datatype;
mod r#enum;
mod generic;
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
pub use generic::*;
pub use object::*;
pub use r#enum::*;
pub use specta_macros::*;
pub use tuple::*;
pub use typescript::*;

pub type TypeDefs = HashMap<&'static str, DataType>;

pub struct DefOpts<'a> {
    pub parent_inline: bool,
    pub type_map: &'a mut TypeDefs,
}

pub trait Type {
    const NAME: &'static str;

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType;
    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType;
    fn definition(opts: DefOpts) -> DataType;
}

macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            const NAME: &'static str = stringify!($i);

            fn inline(_: DefOpts, _: &[DataType]) -> DataType {
                DataType::Primitive(PrimitiveType::$i)
            }

            fn reference(_: DefOpts, _: &[DataType]) -> DataType {
                DataType::Primitive(PrimitiveType::$i)
            }

            fn definition(_: DefOpts) -> DataType {
                panic!()
            }
        }
    )+};
}

#[macro_export]
macro_rules! upsert_def {
    ($opts:ident, $generic:ident, $generics:expr) => {
        <$generic as Type>::inline(
            DefOpts {
                parent_inline: false,
                type_map: $opts.type_map,
            },
            $generics,
        )
        // if let Some(name) = <$generic as Type>::name() {
        //     if let Some(def) = $opts.type_map.get(&name) {
        //         def.clone()
        //     } else {
        //         let def = <$generic as Type>::def(
        //             DefOpts {
        //                 parent_inline: false,
        //                 type_map: $opts.type_map,
        //             },
        //             $generics,
        //         );
        //         $opts.type_map.insert(name, def.clone());
        //         def
        //     }
        // } else {
        //     <$generic as Type>::def(
        //         DefOpts {
        //             parent_inline: false,
        //             type_map: $opts.type_map,
        //         },
        //         $generics,
        //     )
        // }
    };
    ($opts:ident, $generics:expr) => {
        $crate::upsert_def!($opts, T, $generics)
    };
}

macro_rules! impl_tuple {
    (($($i:ident),*)) => {
        impl<$($i: Type + 'static),*> Type for ($($i),*) {
            const NAME: &'static str = "Tuple";

            fn inline(_opts: DefOpts, [$($i),* ..]: &[DataType]) -> DataType {
                DataType::Tuple(TupleType {
                    name: stringify!(($($i),*)).to_string(),
                    fields: vec![$($crate::upsert_def!(_opts, $i, $i)),*],
                    generics: vec![]
                })
            }

            fn reference(_opts: DefOpts, generics: &[DataType]) -> DataType {
                Self::inline(_opts, generics)
            }

            fn definition(_opts: DefOpts) -> DataType {
                panic!()
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
// // T = (T1)
// impl_tuple!((T1, T2));
// impl_tuple!((T1, T2, T3));
// impl_tuple!((T1, T2, T3, T4));
// impl_tuple!((T1, T2, T3, T4, T5));
// impl_tuple!((T1, T2, T3, T4, T5, T6));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11));
// impl_tuple!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12));
//
impl<'a> Type for &'a str {
    const NAME: &'static str = String::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(defs, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::reference(opts, generics)
    }

    fn definition(_: DefOpts) -> DataType {
        panic!()
    }
}

impl<'a, T: Type + 'static> Type for &'a T {
    const NAME: &'static str = T::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(defs, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        T::reference(opts, generics)
    }

    fn definition(opts: DefOpts) -> DataType {
        T::definition(opts)
    }
}

impl<T: Type> Type for Vec<T> {
    const NAME: &'static str = "Vec";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::List(Box::new(generics.get(0).cloned().unwrap_or(T::inline(
            DefOpts {
                parent_inline: false,
                type_map: opts.type_map,
            },
            generics,
        ))))
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::List(Box::new(generics.get(0).cloned().unwrap_or(T::reference(
            DefOpts {
                parent_inline: false,
                type_map: opts.type_map,
            },
            generics,
        ))))
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
    }
}

// impl<'a, T: Type> Type for &'a [T] {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::List(Box::new(upsert_def!(defs)))
//     }
//
//     fn name() -> Option<String> {
//         T::name()
//     }
// }
//
// impl<'a, const N: usize, T: Type> Type for [T; N] {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::List(Box::new(upsert_def!(defs)))
//     }
//
//     fn name() -> Option<String> {
//         T::name()
//     }
// }
//
impl<T: Type> Type for Option<T> {
    const NAME: &'static str = "Option";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Nullable(Box::new(generics.get(0).cloned().unwrap_or(T::inline(
            DefOpts {
                parent_inline: false,
                type_map: opts.type_map,
            },
            generics,
        ))))
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Nullable(Box::new(generics.get(0).cloned().unwrap_or(T::reference(
            DefOpts {
                parent_inline: false,
                type_map: opts.type_map,
            },
            generics,
        ))))
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
    }
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type> Type for $container<T> {
            const NAME: &'static str = stringify!($container);

            fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
                generics.get(0).cloned().unwrap_or(T::inline(
                    DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map,
                    },
                    generics,
                ))
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
                generics.get(0).cloned().unwrap_or(T::reference(
                    DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map,
                    },
                    generics,
                ))
            }

            fn definition(_: DefOpts) -> DataType {
                panic!()
            }
        }
    )+}
}

impl_containers!(Box Rc Arc Cell RefCell Mutex);
//
// impl<T: Type> Type for HashSet<T> {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::List(Box::new(upsert_def!(defs)))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// impl<T: Type> Type for BTreeSet<T> {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::List(Box::new(upsert_def!(defs)))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
impl<K: Type, V: Type> Type for HashMap<K, V> {
    const NAME: &'static str = "HashMap";

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        let k_gen = generics.get(0).cloned().unwrap_or(<K as Type>::inline(
            DefOpts {
                parent_inline: false,
                type_map: defs.type_map,
            },
            &[],
        ));
        let v_gen = generics.get(1).cloned().unwrap_or(<V as Type>::inline(
            DefOpts {
                parent_inline: false,
                type_map: defs.type_map,
            },
            &[],
        ));

        DataType::Record(Box::new((
            K::inline(
                DefOpts {
                    parent_inline: false,
                    type_map: defs.type_map,
                },
                &[k_gen],
            ),
            V::inline(
                DefOpts {
                    parent_inline: false,
                    type_map: defs.type_map,
                },
                &[v_gen],
            ),
        )))
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Record(Box::new((
            generics.get(0).cloned().unwrap_or(K::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map,
                },
                generics,
            )),
            generics.get(1).cloned().unwrap_or(V::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map,
                },
                generics,
            )),
        )))
    }

    fn definition(_: DefOpts) -> DataType {
        panic!()
    }
}
//
// impl<K: Type, V: Type> Type for BTreeMap<K, V> {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::Record(Box::new((
//             K::def(DefOpts {
//                 parent_inline: false,
//                 type_map: defs.type_map,
//             }),
//             V::def(DefOpts {
//                 parent_inline: false,
//                 type_map: defs.type_map,
//             }),
//         )))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// #[cfg(feature = "indexmap")]
// impl<T: Type> Type for indexmap::IndexSet<T> {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::List(Box::new(upsert_def!(defs)))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// #[cfg(feature = "indexmap")]
// impl<K: Type, V: Type> Type for indexmap::IndexMap<K, V> {
//     fn def(defs: &mut TypeDefs) -> DataType {
//         DataType::Record(Box::new((K::def(defs), V::def(defs))))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// #[cfg(feature = "serde")]
// impl<K: Type, V: Type> Type for serde_json::Map<K, V> {
//     fn def(defs: DefOpts) -> DataType {
//         DataType::Record(Box::new((
//             K::def(DefOpts {
//                 parent_inline: false,
//                 type_map: defs.type_map,
//             }),
//             V::def(DefOpts {
//                 parent_inline: false,
//                 type_map: defs.type_map,
//             }),
//         )))
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// #[cfg(feature = "serde")]
// impl Type for serde_json::Value {
//     fn def(_defs: DefOpts) -> DataType {
//         DataType::Any
//     }
//
//     fn name() -> Option<String> {
//         None
//     }
// }
//
// #[cfg(feature = "uuid")]
// impl Type for uuid::Uuid {
//     fn def(defs: DefOpts) -> DataType {
//         String::def(defs)
//     }
//
//     fn name() -> Option<String> {
//         String::name()
//     }
// }
//
// #[cfg(feature = "chrono")]
// impl<T: chrono::TimeZone> Type for chrono::DateTime<T> {
//     fn def(defs: DefOpts) -> DataType {
//         String::def(defs)
//     }
//
//     fn name() -> Option<String> {
//         String::name()
//     }
// }
