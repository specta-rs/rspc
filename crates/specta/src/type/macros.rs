macro_rules! impl_primitives {
    ($($i:ident)+) => {$(
        impl Type for $i {
            const NAME: &'static str = stringify!($i);
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

            fn inline(_: DefOpts, _: &[DataType]) -> DataType {
                DataType::Primitive(datatype::PrimitiveType::$i)
            }
        }
    )+};
}

macro_rules! impl_tuple {
    ( impl $i:ident ) => {
        impl_tuple!(impl); // This does tuple struct
    }; // T = (T1)
    ( impl $($i:ident),* ) => {
        #[allow(non_snake_case)]
        impl<$($i: Type + 'static),*> Type for ($($i),*) {
            const NAME: &'static str = stringify!(($($i::NAME),*));
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

            fn inline(_opts: DefOpts, generics: &[DataType]) -> DataType {
                let mut _generics = generics.iter();

                $(let $i = _generics.next().map(Clone::clone).unwrap_or_else(|| $i::reference(
                    DefOpts {
                        parent_inline: _opts.parent_inline,
                        type_map: _opts.type_map
                    }, &[]
                ));)*

                DataType::Tuple(datatype::TupleType {
                    name: <Self as Type>::NAME,
                    fields: vec![$($i),*],
                    generics: vec![]
                })
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_tuple!(impl $i2 $(, $i)* );
        impl_tuple!($($i),*);
    };
    () => {};
}

macro_rules! impl_containers {
    ($($container:ident)+) => {$(
        impl<T: Type> Type for $container<T> {
            const NAME: &'static str = stringify!($container);
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

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
        }
    )+}
}

macro_rules! impl_as {
    ($($ty:path as $tty:ident)+) => {$(
        impl Type for $ty {
            const NAME: &'static str = stringify!($ty);
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

            fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
                <$tty as Type>::inline(opts, generics)
            }

            fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
                <$tty as Type>::reference(opts, generics)
            }
        }
    )+};
}

macro_rules! impl_for_list {
    ($($ty:path as $name:expr)+) => {$(
        impl<T: Type> Type for $ty {
            const NAME: &'static str = $name;
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

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
        }
    )+};
}

macro_rules! impl_for_map {
    ($ty:path as $name:expr) => {
        impl<K: Type, V: Type> Type for $ty {
            const NAME: &'static str = $name;
            const SID: $crate::TypeSid = $crate::sid!();
            const IMPL_LOCATION: $crate::ImplLocation = $crate::impl_location!();

            fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
                DataType::Record(Box::new((
                    generics.get(0).cloned().unwrap_or(<K as Type>::inline(
                        DefOpts {
                            parent_inline: false,
                            type_map: defs.type_map,
                        },
                        &[],
                    )),
                    generics.get(1).cloned().unwrap_or(<V as Type>::inline(
                        DefOpts {
                            parent_inline: false,
                            type_map: defs.type_map,
                        },
                        &[],
                    )),
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
        }
    };
}
