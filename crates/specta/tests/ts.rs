use std::{
    cell::RefCell,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
};

use serde::Serialize;
use specta::Type;

macro_rules! assert_ts {
    ($t:ty, $e:expr) => {
        assert_eq!(specta::ts::inline::<$t>(&Default::default()).unwrap(), $e)
    };
}
pub(crate) use assert_ts;

macro_rules! assert_ts_export {
    ($t:ty, $e:expr) => {
        assert_eq!(specta::ts::export::<$t>(&Default::default()).unwrap(), $e)
    };
}
pub(crate) use assert_ts_export;

// TODO: Unit test other `specta::Type` methods such as `::reference(...)`

#[test]
fn typescript_types() {
    assert_ts!(i8, "number");
    assert_ts!(u8, "number");
    assert_ts!(i16, "number");
    assert_ts!(u16, "number");
    assert_ts!(i32, "number");
    assert_ts!(u32, "number");
    assert_ts!(f32, "number");
    assert_ts!(f64, "number");

    assert_ts!(bool, "boolean");

    assert_ts!((), "null");
    assert_ts!((String, i32), "[string, number]");
    assert_ts!((String, i32, bool), "[string, number, boolean]");
    assert_ts!(
        (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool),
        "[boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean]"
    );

    assert_ts!(String, "string");
    // impossible since Path as a generic is unsized lol
    // assert_ts!(Path, "string");
    assert_ts!(PathBuf, "string");
    assert_ts!(IpAddr, "string");
    assert_ts!(Ipv4Addr, "string");
    assert_ts!(Ipv6Addr, "string");
    assert_ts!(SocketAddr, "string");
    assert_ts!(SocketAddrV4, "string");
    assert_ts!(SocketAddrV6, "string");
    assert_ts!(char, "string");
    assert_ts!(&'static str, "string");

    assert_ts!(&'static bool, "boolean");
    assert_ts!(&'static i32, "number");

    assert_ts!(Vec<i32>, "number[]");
    assert_ts!(&[i32], "number[]");
    assert_ts!(&[i32; 5], "number[]");

    assert_ts!(Option<i32>, "number | null");

    assert_ts!(Unit1, "null");
    assert_ts!(Unit2, "null");
    assert_ts!(Unit3, "null");

    assert_ts!(
        SimpleStruct,
        "{ a: number; b: string; c: [number, string, number]; d: string[]; e: string | null }"
    );
    assert_ts!(TupleStruct1, "number");
    assert_ts!(TupleStruct3, "[number, boolean, string]");

    assert_ts!(
        TestEnum,
        r#""Unit" | { Single: number } | { Multiple: [number, number] } | { Struct: { a: number } }"#
    );
    assert_ts!(RefStruct, "TestEnum");

    assert_ts!(
        InlinerStruct,
        "{ inline_this: { ref_struct: SimpleStruct; val: number }; dont_inline_this: RefStruct }"
    );

    assert_ts!(GenericStruct<i32>, "{ arg: number }");
    assert_ts!(GenericStruct<String>, "{ arg: string }");

    assert_ts!(
        FlattenEnumStruct,
        r#"({ tag: "One" } | { tag: "Two" } | { tag: "Three" }) & { outer: string }"#
    );

    assert_ts!(OverridenStruct, "{ overriden_field: string }");
    assert_ts!(HasGenericAlias, r#"{ [key: number]: string }"#);
}

#[derive(Type)]
struct Unit1;

#[derive(Type)]
struct Unit2 {}

#[derive(Type)]
struct Unit3();

#[derive(Type)]
struct SimpleStruct {
    a: i32,
    b: String,
    c: (i32, String, RefCell<i32>),
    d: Vec<String>,
    e: Option<String>,
}

#[derive(Type)]
struct TupleStruct1(i32);

#[derive(Type)]
struct TupleStruct3(i32, bool, String);

#[derive(Type)]
#[specta(rename = "HasBeenRenamed")]
struct RenamedStruct;

#[derive(Type)]
enum TestEnum {
    Unit,
    Single(i32),
    Multiple(i32, i32),
    Struct { a: i32 },
}

#[derive(Type)]
struct RefStruct(TestEnum);

#[derive(Type)]
struct InlineStruct {
    ref_struct: SimpleStruct,
    val: i32,
}

#[derive(Type)]
struct InlinerStruct {
    #[specta(inline)]
    inline_this: InlineStruct,
    dont_inline_this: RefStruct,
}

#[derive(Type)]
struct GenericStruct<T> {
    arg: T,
}

#[derive(Serialize, Type)]
struct FlattenEnumStruct {
    outer: String,
    #[serde(flatten)]
    inner: FlattenEnum,
}

#[derive(Serialize, Type)]
#[serde(tag = "tag", content = "test")]
enum FlattenEnum {
    One,
    Two,
    Three,
}

#[derive(Serialize, Type)]
struct OverridenStruct {
    #[specta(type = String)]
    overriden_field: i32,
}

#[derive(Type)]
struct HasGenericAlias(GenericAlias<i32>);

type GenericAlias<T> = std::collections::HashMap<T, String>;
