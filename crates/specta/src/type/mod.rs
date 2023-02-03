use crate::*;

#[macro_use]
mod macros;
mod impls;

/// The category a type falls under. Determines how references are generated for a given type.
pub enum TypeCategory {
    /// No references should be created, instead just copies the inline representation of the type.
    Inline(DataType),
    /// The type should be properly referenced and stored in the type map to be defined outside of
    /// where it is referenced.
    Reference {
        /// Datatype to be put in the type map while field types are being resolved. Used in order to
        /// support recursive types without causing an infinite loop.
        ///
        /// This works since a child type that references a parent type does not care about the
        /// parent's fields, only really its name. Once all of the parent's fields have been
        /// resolved will the parent's definition be placed in the type map.
        ///
        /// This doesn't account for flattening and inlining recursive types, however, which will
        /// require a more complex solution since it will require multiple processing stages.
        placeholder: DataType,
        /// Datatype to use whenever a reference to the type is requested.
        reference: DataType,
    },
}

/// Provides runtime type information that can be fed into a language exporter to generate a type definition in another language.
/// Avoid implementing this trait yourself where possible and use the [`Type`](derive@crate::Type) macro instead.
pub trait Type {
    /// The name of the type
    const NAME: &'static str;

    /// Rust documentation comments on the type
    const COMMENTS: &'static [&'static str] = &[];

    /// The Specta ID for the type. The value for this should come from the `sid!();` macro.
    const SID: TypeSid;

    /// The code location where this type is implemented. Used for error reporting.
    const IMPL_LOCATION: ImplLocation;

    /// Returns the inline definition of a type with generics substituted for those provided.
    /// This function defines the base structure of every type, and is used in both
    /// [`definition`](crate::Type::definition) and [`reference`](crate::Type::definition)
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType;

    /// Returns the type parameter generics of a given type.
    /// Will usually be empty except for custom types.
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn definition_generics() -> Vec<GenericType> {
        vec![]
    }

    /// Small wrapper around [`inline`](crate::Type::inline) that provides
    /// [`definition_generics`](crate::Type::definition_generics)
    /// as the value for the `generics` arg.
    ///
    /// Implemented internally
    fn definition(opts: DefOpts) -> DataTypeExt {
        DataTypeExt {
            name: Self::NAME,
            comments: Self::COMMENTS,
            sid: Self::SID,
            impl_location: Self::IMPL_LOCATION,
            inner: Self::inline(
                opts,
                &Self::definition_generics()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
            ),
        }
    }

    /// Defines which category this type falls into, determining how references to it are created.
    /// See [`TypeCategory`] for more info.
    ///
    /// Implemented internally or via the [`Type`](derive@crate::Type) macro
    fn category_impl(opts: DefOpts, generics: &[DataType]) -> TypeCategory {
        TypeCategory::Inline(Self::inline(opts, generics))
    }

    /// Generates a datatype corresponding to a reference to this type,
    /// as determined by its category. Getting a reference to a type implies that
    /// it should belong in the type map (since it has to be referenced from somewhere),
    /// so the output of [`definition`](crate::Type::definition) will be put into the type map.
    ///
    /// Implemented internally
    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        let category = Self::category_impl(
            DefOpts {
                parent_inline: false,
                type_map: opts.type_map,
            },
            generics,
        );

        match category {
            TypeCategory::Inline(inline) => inline,
            TypeCategory::Reference {
                placeholder,
                reference,
            } => {
                opts.type_map.entry(Self::NAME).or_insert(DataTypeExt {
                    name: Self::NAME,
                    comments: Self::COMMENTS,
                    sid: Self::SID,
                    impl_location: Self::IMPL_LOCATION,
                    inner: placeholder,
                });

                let definition = Self::definition(DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map,
                });

                if let Some(ty) = opts.type_map.get(&Self::NAME) {
                    // TODO: Properly detect duplicate name where SID don't match
                    // println!("{:#?} {:?}", ty, definition);
                    if matches!(ty.inner, DataType::Placeholder) {
                        opts.type_map.insert(Self::NAME, definition);
                    } else {
                        if ty.sid != definition.sid {
                            // TODO: Return runtime error instead of panicking
                            panic!("Specta: you have tried to export two types both called '{}' declared at '{}' and '{}'! You could give both types a unique name or put `#[specta(inline)]` on one/both of them to cause it to be exported without a name.", ty.name, ty.impl_location.as_str(), definition.impl_location.as_str());
                        }
                    }
                } else {
                    opts.type_map.insert(Self::NAME, definition);
                }

                reference
            }
        }
    }
}

/// A marker trait for compile-time validation of which types can be flattened.
pub trait Flatten: Type {}

/// The Specta ID for the type. Holds for the given properties `T::SID == T::SID`, `T::SID != S::SID` and `Type<T>::SID == Type<S>::SID` (unlike std::any::TypeId)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeSid(u64);

/// Compute an SID hash for a given type.
/// This hash function comes from https://stackoverflow.com/a/71464396
/// You should NOT use this directly. Rely on `sid!();` instead.
#[doc(hidden)]
pub const fn internal_sid_hash(
    module_path: &'static str,
    file: &'static str,
    // This is required for a unique hash because all impls generated by a `macro_rules!` will have an identical `module_path` and `file` value.
    type_name: &'static str,
) -> TypeSid {
    let mut hash = 0xcbf29ce484222325;
    let prime = 0x00000100000001B3;

    let mut bytes = module_path.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    bytes = file.as_bytes();
    i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    bytes = type_name.as_bytes();
    i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    TypeSid(hash)
}

/// Compute an SID hash for a given type.
#[macro_export]
macro_rules! sid {
    () => {
        $crate::internal_sid_hash(
            module_path!(),
            <Self as $crate::Type>::IMPL_LOCATION.as_str(),
            <Self as $crate::Type>::NAME,
        )
    };
     // Using `$crate_path:path` here does not work because: https://github.com/rust-lang/rust/issues/48067
    (@with_specta_path; $first:ident$(::$rest:ident)*) => {{
        use $first$(::$rest)*::{internal_sid_hash, Type};

        internal_sid_hash(
            module_path!(),
            <Self as Type>::IMPL_LOCATION.as_str(),
            <Self as Type>::NAME,
        )
    }};
}

/// The location of the impl block for a given type. This is used for error reporting.
/// The content of it is transparent and should be generated by the `impl_location!` macro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplLocation(&'static str);

impl ImplLocation {
    #[doc(hidden)]
    pub const fn internal_new(s: &'static str) -> Self {
        Self(s)
    }

    /// Get the location as a string
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

/// Compute the location for an impl block
#[macro_export]
macro_rules! impl_location {
    () => {
        $crate::ImplLocation::internal_new(concat!(file!(), ":", line!(), ":", column!()))
    };
    // Using `$crate_path:path` here does not work because: https://github.com/rust-lang/rust/issues/48067
    (@with_specta_path; $first:ident$(::$rest:ident)*) => {
        $first$(::$rest)*::ImplLocation::internal_new(concat!(file!(), ":", line!(), ":", column!()))
    };
}
