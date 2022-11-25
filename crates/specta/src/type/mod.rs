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
    fn definition(opts: DefOpts) -> DataType {
        Self::inline(
            opts,
            &Self::definition_generics()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        )
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
                if !opts.type_map.contains_key(Self::NAME) {
                    opts.type_map.insert(Self::NAME, placeholder);

                    let definition = Self::definition(DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map,
                    });

                    opts.type_map.insert(Self::NAME, definition);
                }

                reference
            }
        }
    }
}

/// A marker trait for compile-time validation of which types can be flattened.
pub trait Flatten: Type {}
