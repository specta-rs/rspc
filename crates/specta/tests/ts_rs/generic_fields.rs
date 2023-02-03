#![allow(dead_code, clippy::box_collection)]

use std::borrow::Cow;

use crate::ts::assert_ts;
use specta::Type;

#[test]
fn newtype() {
    #[derive(Type)]
    struct Newtype1(Vec<Cow<'static, i32>>);
    assert_ts!(Newtype1, "number[]");
}

#[test]
fn newtype_nested() {
    #[derive(Type)]
    struct Newtype2(Vec<Vec<i32>>);
    assert_ts!(Newtype2, "number[][]");
}

#[test]
fn alias() {
    type Alias1 = Vec<String>;
    assert_ts!(Alias1, "string[]");
}

#[test]
fn alias_nested() {
    type Alias2 = Vec<Vec<String>>;
    assert_ts!(Alias2, "string[][]");
}

#[test]
fn named() {
    #[derive(Type)]
    struct Struct1 {
        a: Box<Vec<String>>,
        b: (Vec<String>, Vec<String>),
        c: [Vec<String>; 3],
    }
    assert_ts!(
        Struct1,
        "{ a: string[]; b: [string[], string[]]; c: string[][] }"
    );
}

#[test]
fn named_nested() {
    #[derive(Type)]
    struct Struct2 {
        a: Vec<Vec<String>>,
        b: (Vec<Vec<String>>, Vec<Vec<String>>),
        c: [Vec<Vec<String>>; 3],
    }
    assert_ts!(
        Struct2,
        "{ a: string[][]; b: [string[][], string[][]]; c: string[][][] }"
    );
}

#[test]
fn tuple() {
    #[derive(Type)]
    struct Tuple1(Vec<i32>, (Vec<i32>, Vec<i32>), [Vec<i32>; 3]);
    assert_ts!(Tuple1, "[number[], [number[], number[]], number[][]]");
}

#[test]
fn tuple_nested() {
    #[derive(Type)]
    struct Tuple2(
        Vec<Vec<i32>>,
        (Vec<Vec<i32>>, Vec<Vec<i32>>),
        [Vec<Vec<i32>>; 3],
    );
    assert_ts!(
        Tuple2,
        "[number[][], [number[][], number[][]], number[][][]]"
    );
}
