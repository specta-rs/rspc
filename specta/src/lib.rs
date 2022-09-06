#![forbid(unsafe_code)]

mod datatype;
mod r#enum;
pub mod impl_type_macros;
mod object;
mod tuple;
mod typescript;

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
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

pub type TypeDefs = BTreeMap<&'static str, DataType>;

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

pub trait Flatten: Type {
    fn flatten(opts: DefOpts, generics: &[DataType]) -> Vec<ObjectField> {
        match Self::inline(opts, generics) {
            DataType::Object(ObjectType { fields, .. }) => fields,
            _ => unreachable!(
                "Type '{}' implements flatten but is not an object!",
                Self::NAME
            ),
        }
    }
}

impl_primitives!(
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
    f32 f64
    bool char
    String
    Path
    PathBuf
    IpAddr Ipv4Addr Ipv6Addr
    SocketAddr SocketAddrV4 SocketAddrV6
);

impl_containers!(Box Rc Arc Cell RefCell Mutex);

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
    const NAME: &'static str = String::NAME;

    fn inline(defs: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(defs, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::reference(opts, generics)
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
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

impl_as!(str as String);

impl_for_list!(Vec<T>, "Vec");
impl_for_list!(HashSet<T>, "HashSet");
impl_for_list!(BTreeSet<T>, "BTreeSet");

impl<'a, T: Type> Type for &'a [T] {
    const NAME: &'static str = "&[T]";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::inline(opts, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::reference(opts, generics)
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
    }
}

impl<'a, const N: usize, T: Type> Type for [T; N] {
    const NAME: &'static str = "&[T; N]";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::inline(opts, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        <Vec<T>>::reference(opts, generics)
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
    }
}

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

impl_for_map!(HashMap<K, V>, "HashMap");
impl_for_map!(BTreeMap<K, V>, "BTreeMap");

#[cfg(feature = "indexmap")]
impl_for_list!(indexmap::IndexSet<T>, "IndexSet");

#[cfg(feature = "indexmap")]
impl_for_map!(indexmap::IndexMap<K, V>, "IndexMap");

#[cfg(feature = "serde")]
impl_for_map!(serde_json::Map<K, V>, "Map");

#[cfg(feature = "serde")]
impl Type for serde_json::Value {
    const NAME: &'static str = "Value";

    fn inline(_: DefOpts, _: &[DataType]) -> DataType {
        DataType::Any
    }

    fn reference(_: DefOpts, _: &[DataType]) -> DataType {
        DataType::Any
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
    }
}

#[cfg(feature = "uuid")]
impl_as!(uuid::Uuid as String);

#[cfg(feature = "uuid")]
impl_as!(uuid::fmt::Hyphenated as String);

#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> Type for chrono::DateTime<T> {
    const NAME: &'static str = "DateTime";

    fn inline(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::inline(opts, generics)
    }

    fn reference(opts: DefOpts, generics: &[DataType]) -> DataType {
        String::reference(opts, generics)
    }

    fn definition(opts: DefOpts) -> DataType {
        String::definition(opts)
    }
}

#[cfg(feature = "chrono")]
impl_as!(chrono::NaiveDateTime as String);

#[cfg(feature = "chrono")]
impl_as!(chrono::NaiveDate as String);

#[cfg(feature = "chrono")]
impl_as!(chrono::NaiveTime as String);

#[cfg(feature = "time")]
impl_as!(time::PrimitiveDateTime as String);

#[cfg(feature = "time")]
impl_as!(time::OffsetDateTime as String);

#[cfg(feature = "time")]
impl_as!(time::Date as String);

#[cfg(feature = "time")]
impl_as!(time::Time as String);

#[cfg(feature = "bigdecimal")]
impl_as!(bigdecimal::BigDecimal as String);

// This assumes the `serde-with-str` feature is enabled. Check #26 for more info.
#[cfg(feature = "rust_decimal")]
impl_as!(rust_decimal::Decimal as String);

#[cfg(feature = "ipnetwork")]
impl_as!(ipnetwork::IpNetwork as String);

#[cfg(feature = "ipnetwork")]
impl_as!(ipnetwork::Ipv4Network as String);

#[cfg(feature = "ipnetwork")]
impl_as!(ipnetwork::Ipv6Network as String);

#[cfg(feature = "mac_address")]
impl_as!(mac_address::MacAddress as String);

#[cfg(feature = "chrono")]
impl_as!(chrono::FixedOffset as String);

#[cfg(feature = "chrono")]
impl_as!(chrono::Utc as String);

#[cfg(feature = "chrono")]
impl_as!(chrono::Local as String);
