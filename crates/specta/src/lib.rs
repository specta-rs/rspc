//! Easily export your Rust types to other languages
//!
//! Specta provides a system for type introspection and a set of language exporter which allows you to export your Rust types to other languages! Currently we support exporting to [Typescript](https://www.typescriptlang.org) and have alpha support for [OpenAPI](https://www.openapis.org).
//!
//! ## Example
//! ```rust
//! use specta::{export_fn, ts::{ts_export_datatype, ts_export}, ToDataType, Type};
//!
//! #[derive(Type)]
//! pub struct MyCustomType {
//!    pub my_field: String,
//! }
//!
//! #[specta::command]
//! fn some_function(name: String, age: i32) -> bool {
//!     true
//! }
//!
//! fn main() {
//!     assert_eq!(
//!         ts_export::<MyCustomType>(),
//!         Ok("export interface MyCustomType { my_field: string }".to_string())
//!         
//!     );
//!
//!      // This API is pretty new and will likely under go API changes in the future.
//!      assert_eq!(
//!         ts_export_datatype(&export_fn!(some_function).to_data_type()),
//!         Ok("export interface CommandDataType { name: \"some_function\", input: { name: string, age: number }, result: boolean }".to_string())
//!      );
//! }
//! ```
//!
//! ## Known limitations
//!  - Type aliases must not alias generic types (as far as known this is just a Rust limitation)
//!
//! ## Why not ts-rs?
//!
//! ts-rs is a great library, but it has a few limitations which became a problem when I was building [rspc](https://github.com/oscartbeaumont/rspc). Namely it deals with types individually which means it is not possible to export a type and all of the other types it depends on.
//!
//! ## Supported Libraries
//!
//! If you are using [Prisma Client Rust](https://prisma.brendonovich.dev) you can enable the `rspc` feature on it to allow for Specta support on types coming directly from your database. This includes support for the types created via a selection.
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::unwrap_used, clippy::panic, missing_docs)]

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    ffi::{CStr, CString, OsStr, OsString},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
            AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        RwLock,
    },
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

/// Support for Specta commands. These allow exporting the types for Rust functions.
#[cfg(feature = "command")]
pub mod command;
/// Types related to working with [`crate::DataType`] directly.
/// This is for advanced users.
pub mod datatype;
pub mod export;
/// Types to represent the structure of the Rust types for the type exporters.
pub mod r#type;
#[macro_use]
mod impl_type_macros;
mod lang;
mod to_data_type;

// #[cfg(feature = "command")]
// pub use command::*;
use datatype::*;
pub use lang::*;
use r#type::{DefOpts, TypeDefs};

/// Derive type is used to derive the [`Type`](crate::Type) trait on a struct.
///
/// ## Example
///
/// ```rust
/// use specta::Type;
///
/// // Use it on structs
/// #[derive(Type)]
/// pub struct MyCustomStruct {
///     pub name: String,
/// }
///
/// #[derive(Type)]
/// pub struct MyCustomStruct2(String, i32, bool);
///
/// // Use it on enums
/// #[derive(Type)]
/// pub enum MyCustomType {
///     VariantOne,
///     VariantTwo(String, i32),
///     VariantThree { name: String, age: i32 },
/// }
/// ```
pub use specta_macros::Type;

#[doc(hidden)]
/// This macro is exposed from rspc as a wrapper around [Type] with a correct import path.
/// This is exposed from here so rspc doesn't need a macro package for 4 lines of code.
pub use specta_macros::RSPCType;

/// Derive command is used to derive the [`ToDataType`](crate::ToDataType) trait on a struct.
///
/// This is designed a more advanced feature. If you are just looking for regular type exporting use [`Type`](derive@crate::Type) instead.
///
///
/// ## Example
///
/// ```rust
/// use specta::{
///     datatype::{DataType, LiteralType},
///     ts::ts_export_datatype,
///     ToDataType,
/// };
///
/// #[derive(ToDataType)]
/// pub struct MyEnum(pub Vec<DataType>);
///
/// fn main() {
///     let e = MyEnum(vec![
///         DataType::Literal(LiteralType::String("A".to_string())),
///         DataType::Literal(LiteralType::String("B".to_string())),
///     ]);
///
///     assert_eq!(
///         ts_export_datatype(&e.to_data_type()).unwrap(),
///         "export type MyEnum = \"A\" | \"B\""
///     );
/// }
/// ```
///
pub use specta_macros::ToDataType;

/// Attribute macro which can be put on a Rust function to introspect its types.
///
/// ```rust
/// #[specta::command]
/// fn my_function(arg1: i32, arg2: bool) -> &'static str {
///     "Hello World"
/// }
/// ```
pub use specta_macros::command;

pub use to_data_type::*;

#[doc(hidden)]
pub mod internal {
    pub use ctor;
    pub use paste::paste as _specta_paste;
}

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

/// A trait which allows runtime type reflection of a type it is implemented on.
/// The type information can then be fed into a language exporter to generate a type definition in another language.
/// You should avoid implementing this trait yourself where possible and use the [`Type`](derive@crate::Type) macro instead.
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
                .map(ToDataType::to_data_type)
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

impl<K: Type, V: Type> Flatten for std::collections::HashMap<K, V> {}
impl<K: Type, V: Type> Flatten for std::collections::BTreeMap<K, V> {}

impl_primitives!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64
    bool char
    String
);

impl_containers!(Box Rc Arc Cell RefCell Mutex RwLock);

impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

impl<'a> Type for &'a str {
    const NAME: &'static str = String::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(defs, generics)
    }
}

impl<'a, T: Type + 'static> Type for &'a T {
    const NAME: &'static str = T::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(defs, generics)
    }
}

impl<'a, T: ToOwned + Type + 'static> Type for Cow<'a, T> {
    const NAME: &'static str = T::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        T::inline(defs, generics)
    }
}

impl_as!(
    str as String
    CString as String
    CStr as String
    OsString as String
    OsStr as String

    Path as String
    PathBuf as String

    IpAddr as String
    Ipv4Addr as String
    Ipv6Addr as String

    SocketAddr as String
    SocketAddrV4 as String
    SocketAddrV6 as String

    SystemTime as String
    Instant as String
    Duration as String

    AtomicBool as bool
    AtomicI8 as i8
    AtomicI16 as i16
    AtomicI32 as i32
    AtomicIsize as isize
    AtomicU8 as u8
    AtomicU16 as u16
    AtomicU32 as u32
    AtomicUsize as usize
    AtomicI64 as i64
    AtomicU64 as u64

    NonZeroU8 as u8
    NonZeroU16 as u16
    NonZeroU32 as u32
    NonZeroU64 as u64
    NonZeroUsize as usize
    NonZeroI8 as i8
    NonZeroI16 as i16
    NonZeroI32 as i32
    NonZeroI64 as i64
    NonZeroIsize as isize
    NonZeroU128 as u128
    NonZeroI128 as i128
);

impl_for_list!(
    Vec<T> as "Vec"
    VecDeque<T> as "VecDeque"
    BinaryHeap<T> as "BinaryHeap"
    LinkedList<T> as "LinkedList"
    HashSet<T> as "HashSet"
    BTreeSet<T> as "BTreeSet"
);

impl<'a, T: Type> Type for &'a [T] {
    const NAME: &'static str = "&[T]";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::inline(opts, generics)
    }

    fn category_impl(opts: DefOpts, generics: &[DataType]) -> TypeCategory {
        <Vec<T>>::category_impl(opts, generics)
    }
}

impl<const N: usize, T: Type> Type for [T; N] {
    const NAME: &'static str = "&[T; N]";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::inline(opts, generics)
    }

    fn category_impl(opts: DefOpts, generics: &[DataType]) -> TypeCategory {
        <Vec<T>>::category_impl(opts, generics)
    }
}

impl<T: Type> Type for Option<T> {
    const NAME: &'static str = "Option";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        DataType::Nullable(Box::new(generics.get(0).cloned().unwrap_or_else(|| {
            T::inline(
                DefOpts {
                    parent_inline: false,
                    type_map: opts.type_map,
                },
                generics,
            )
        })))
    }

    fn category_impl(opts: DefOpts, generics: &[DataType]) -> TypeCategory {
        TypeCategory::Inline(DataType::Nullable(Box::new(
            generics.get(0).cloned().unwrap_or_else(|| {
                T::reference(
                    DefOpts {
                        parent_inline: false,
                        type_map: opts.type_map,
                    },
                    generics,
                )
            }),
        )))
    }
}

impl_for_map!(HashMap<K, V> as "HashMap");
impl_for_map!(BTreeMap<K, V> as "BTreeMap");

#[cfg(feature = "indexmap")]
impl_for_list!(indexmap::IndexSet<T> as "IndexSet");

#[cfg(feature = "indexmap")]
impl_for_map!(indexmap::IndexMap<K, V> as "IndexMap");

#[cfg(feature = "serde")]
impl_for_map!(serde_json::Map<K, V> as "Map");

#[cfg(feature = "serde")]
impl Type for serde_json::Value {
    const NAME: &'static str = "Value";

    fn inline(_: DefOpts, _: &[DataType]) -> DataType {
        DataType::Any
    }
}

#[cfg(feature = "uuid")]
impl_as!(
    uuid::Uuid as String
    uuid::fmt::Hyphenated as String
);

#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> Type for chrono::DateTime<T> {
    const NAME: &'static str = "DateTime";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(opts, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::reference(opts, generics)
    }
}

#[cfg(feature = "chrono")]
impl_as!(
    chrono::NaiveDateTime as String
    chrono::NaiveDate as String
    chrono::NaiveTime as String
);

#[cfg(feature = "time")]
impl_as!(
    time::PrimitiveDateTime as String
    time::OffsetDateTime as String
    time::Date as String
    time::Time as String
);

#[cfg(feature = "bigdecimal")]
impl_as!(bigdecimal::BigDecimal as String);

// This assumes the `serde-with-str` feature is enabled. Check #26 for more info.
#[cfg(feature = "rust_decimal")]
impl_as!(rust_decimal::Decimal as String);

#[cfg(feature = "ipnetwork")]
impl_as!(
    ipnetwork::IpNetwork as String
    ipnetwork::Ipv4Network as String
    ipnetwork::Ipv6Network as String
);

#[cfg(feature = "mac_address")]
impl_as!(mac_address::MacAddress as String);

#[cfg(feature = "chrono")]
impl_as!(
    chrono::FixedOffset as String
    chrono::Utc as String
    chrono::Local as String
);

#[cfg(feature = "bson")]
impl_as!(
    bson::oid::ObjectId as String
    bson::Decimal128 as i128
    bson::DateTime as String
    bson::Uuid as String
);

// TODO: bson::bson
// TODO: bson::Document

#[cfg(feature = "bytesize")]
impl_as!(bytesize::ByteSize as u64);

#[cfg(feature = "uhlc")]
pub use uhlc_impls::*;

#[cfg(feature = "uhlc")]
mod uhlc_impls {
    use crate::r#type::ObjectType;

    use super::*;
    use std::any::TypeId;
    use uhlc::*;

    impl_as!(
        NTP64 as u64
        ID as NonZeroU128
    );

    impl Type for Timestamp {
        const NAME: &'static str = "Timestamp";

        fn inline(opts: DefOpts, _: &[DataType]) -> DataType {
            use r#type::ObjectField;

            DataType::Object(ObjectType {
                name: "Timestamp".to_string(),
                generics: vec![],
                fields: vec![
                    ObjectField {
                        name: "id".to_string(),
                        optional: false,
                        flatten: false,
                        ty: {
                            let ty = <ID as Type>::reference(
                                DefOpts {
                                    parent_inline: false,
                                    type_map: opts.type_map,
                                },
                                &[],
                            );
                            ty
                        },
                    },
                    ObjectField {
                        name: "time".to_string(),
                        optional: false,
                        flatten: false,
                        ty: {
                            let ty = <NTP64 as Type>::reference(
                                DefOpts {
                                    parent_inline: false,
                                    type_map: opts.type_map,
                                },
                                &[],
                            );
                            ty
                        },
                    },
                ],
                tag: None,
                type_id: Some(TypeId::of::<Self>()),
            })
        }

        fn reference(opts: DefOpts, _: &[DataType]) -> DataType {
            DataType::Reference {
                name: Self::NAME.to_string(),
                generics: vec![],
                type_id: TypeId::of::<Self>(),
            }
        }

        fn placeholder() -> Option<DataType> {
            Some(DataType::Object(ObjectType {
                name: Self::NAME.to_string(),
                generics: vec![],
                fields: vec![],
                tag: None,
                type_id: Some(TypeId::of::<Self>()),
            }))
        }
    }

    impl Flatten for Timestamp {}
}

// TODO: impl Type for Fn()
