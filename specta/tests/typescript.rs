#![allow(dead_code)]

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use specta::{get_ts_type, Type};

macro_rules! assert_ts_type {
    ($t:ty, $e:expr) => {
        assert_eq!(
            get_ts_type(&<$t as Type>::def(&mut Default::default()).body),
            $e
        );
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
    assert_ts_type!(Path, "string");
    assert_ts_type!(PathBuf, "string");
    assert_ts_type!(char, "string");
    assert_ts_type!(&'static str, "string");

    assert_ts_type!(&'static bool, "boolean");
    assert_ts_type!(&'static i32, "number");

    assert_ts_type!(Vec<i32>, "number[]");
    assert_ts_type!(&[i32], "number[]");
    assert_ts_type!(&[i32; 5], "number[]");

    assert_ts_type!(Option<i32>, "number | null");

    assert_ts_type!(Unit, "null");
    assert_ts_type!(Unit2, "null");
    assert_ts_type!(Unit3, "null");

    assert_ts_type!(
        SimpleStruct,
        "{ a: number, b: string, c: [number, string, number], d: string[], e: string | null }"
    );
    assert_ts_type!(TupleStruct1, "number");
    assert_ts_type!(TupleStruct3, "[number, boolean, string]");

    assert_eq!(
        &<RenamedStruct as Type>::def(&mut Default::default()).name,
        "HasBeenRenamed"
    );
    // assert_ts_type!(Wrapper<String>, "string");

    assert_ts_type!(
        TestEnum,
        r#""Unit" | { Single: number } | { Multiple: [number, number] } | { Struct: { a: number } }"#
    );
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

// #[derive(Type)]
// struct Wrapper<T>(T);
