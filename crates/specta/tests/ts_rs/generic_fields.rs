#![allow(dead_code, clippy::box_collection)]

use std::borrow::Cow;

use specta::{ts::inline, Type};

#[test]
fn newtype() {
    #[derive(Type)]
    struct Newtype(Vec<Cow<'static, i32>>);
    assert_eq!(inline::<Newtype>(), "Array<number>");
}

#[test]
fn newtype_nested() {
    #[derive(Type)]
    struct Newtype(Vec<Vec<i32>>);
    assert_eq!(inline::<Newtype>(), "Array<Array<number>>");
}

#[test]
fn alias() {
    type Alias = Vec<String>;
    assert_eq!(inline::<Alias>(), "Array<string>");
}

#[test]
fn alias_nested() {
    type Alias = Vec<Vec<String>>;
    assert_eq!(inline::<Alias>(), "Array<Array<string>>");
}

#[test]
fn named() {
    #[derive(Type)]
    struct Struct {
        a: Box<Vec<String>>,
        b: (Vec<String>, Vec<String>),
        c: [Vec<String>; 3],
    }
    assert_eq!(
        inline::<Struct>(),
        "{ a: Array<string>, b: [Array<string>, Array<string>], c: Array<Array<string>> }"
    );
}

#[test]
fn named_nested() {
    #[derive(Type)]
    struct Struct {
        a: Vec<Vec<String>>,
        b: (Vec<Vec<String>>, Vec<Vec<String>>),
        c: [Vec<Vec<String>>; 3],
    }
    assert_eq!(inline::<Struct>(), "{ a: Array<Array<string>>, b: [Array<Array<string>>, Array<Array<string>>], c: Array<Array<Array<string>>> }");
}

#[test]
fn tuple() {
    #[derive(Type)]
    struct Tuple(Vec<i32>, (Vec<i32>, Vec<i32>), [Vec<i32>; 3]);
    assert_eq!(
        inline::<Tuple>(),
        "[Array<number>, [Array<number>, Array<number>], Array<Array<number>>]"
    );
}

#[test]
fn tuple_nested() {
    #[derive(Type)]
    struct Tuple(
        Vec<Vec<i32>>,
        (Vec<Vec<i32>>, Vec<Vec<i32>>),
        [Vec<Vec<i32>>; 3],
    );
    assert_eq!(
        inline::<Tuple>(),
        "[Array<Array<number>>, [Array<Array<number>>, Array<Array<number>>], Array<Array<Array<number>>>]"
    );
}
