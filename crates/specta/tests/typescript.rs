#![allow(dead_code)]

use std::{
    cell::RefCell,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
};

use serde::Serialize;
use specta::{ts, Type};

macro_rules! assert_ts_type {
    ($t:ty, $e:expr) => {
        assert_eq!(ts::inline::<$t>(), $e)
    };
}

#[test]
fn typescript_types() {
    assert_ts_type!(i8, "number");
    assert_ts_type!(u8, "number");
    assert_ts_type!(i16, "number");
    assert_ts_type!(u16, "number");
    assert_ts_type!(i32, "number");
    assert_ts_type!(u32, "number");
    assert_ts_type!(f32, "number");
    assert_ts_type!(f64, "number");
    assert_ts_type!(usize, "number");
    assert_ts_type!(isize, "number");

    assert_ts_type!(i64, "bigint");
    assert_ts_type!(u64, "bigint");
    assert_ts_type!(i128, "bigint");
    assert_ts_type!(i64, "bigint");
    assert_ts_type!(u128, "bigint");

    assert_ts_type!(bool, "boolean");

    assert_ts_type!((), "null");
    assert_ts_type!((String, i32), "[string, number]");
    assert_ts_type!((String, i32, bool), "[string, number, boolean]");
    assert_ts_type!((bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool, bool), "[boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean, boolean]");

    assert_ts_type!(String, "string");
    // impossible since Path as a generic is unsized lol
    // assert_ts_type!(Path, "string");
    assert_ts_type!(PathBuf, "string");
    assert_ts_type!(IpAddr, "string");
    assert_ts_type!(Ipv4Addr, "string");
    assert_ts_type!(Ipv6Addr, "string");
    assert_ts_type!(SocketAddr, "string");
    assert_ts_type!(SocketAddrV4, "string");
    assert_ts_type!(SocketAddrV6, "string");
    assert_ts_type!(char, "string");
    assert_ts_type!(&'static str, "string");

    assert_ts_type!(&'static bool, "boolean");
    assert_ts_type!(&'static i32, "number");

    assert_ts_type!(Vec<i32>, "Array<number>");
    assert_ts_type!(&[i32], "Array<number>");
    assert_ts_type!(&[i32; 5], "Array<number>");

    assert_ts_type!(Option<i32>, "number | null");

    assert_ts_type!(Unit, "null");
    assert_ts_type!(Unit2, "null");
    assert_ts_type!(Unit3, "null");

    assert_ts_type!(
        SimpleStruct,
        "{ a: number, b: string, c: [number, string, number], d: Array<string>, e: string | null }"
    );
    assert_ts_type!(TupleStruct1, "number");
    assert_ts_type!(TupleStruct3, "[number, boolean, string]");

    assert_ts_type!(
        TestEnum,
        r#""Unit" | { Single: number } | { Multiple: [number, number] } | { Struct: { a: number } }"#
    );
    assert_ts_type!(RefStruct, "TestEnum");

    assert_ts_type!(
        InlinerStruct,
        "{ inline_this: { ref_struct: SimpleStruct, val: number }, dont_inline_this: RefStruct }"
    );

    assert_ts_type!(GenericStruct<i32>, "{ arg: number }");
    assert_ts_type!(GenericStruct<String>, "{ arg: string }");

    assert_ts_type!(
        FlattenEnumStruct,
        r#"({ tag: "One" } | { tag: "Two" } | { tag: "Three" }) & { outer: string }"#
    );

    assert_ts_type!(OverridenStruct, "{ overriden_field: string }");
    assert_ts_type!(HasGenericAlias, r#"Record<number, string>"#);
}

#[derive(Type)]
struct Unit;

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
