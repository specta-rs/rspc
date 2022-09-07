#![forbid(unsafe_code)]

mod datatype;
mod r#enum;
pub mod impl_type_macros;
mod object;
mod tuple;
mod typescript;

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
);

impl_containers!(Box Rc Arc Cell RefCell Mutex RwLock);

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

impl<'a, T: ToOwned + Type + 'static> Type for Cow<'a, T> {
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

    fn reference(_: DefOpts, _: &[DataType]) -> DataType {
        DataType::Any
    }

    fn definition(_: DefOpts) -> DataType {
        unreachable!()
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

    fn definition(opts: DefOpts) -> DataType {
        String::definition(opts)
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
