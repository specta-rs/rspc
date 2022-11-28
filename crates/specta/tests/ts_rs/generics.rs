#![allow(dead_code)]

use std::{
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use specta::{ts::export, Type};

#[derive(Type)]
struct Generic<T: Type> {
    value: T,
    values: Vec<T>,
}

#[derive(Type)]
struct GenericAutoBound<T> {
    value: T,
    values: Vec<T>,
}

#[derive(Type)]
struct GenericAutoBound2<T: PartialEq> {
    value: T,
    values: Vec<T>,
}

#[derive(Type)]
struct Container {
    foo: Generic<u32>,
    bar: Box<HashSet<Generic<u32>>>,
    baz: Box<BTreeMap<String, Rc<Generic<String>>>>,
}

#[test]
fn test() {
    assert_eq!(
        export::<Generic<()>>().unwrap(),
        "export type Generic<T> = { value: T, values: Array<T> }"
    );

    assert_eq!(
        export::<GenericAutoBound::<()>>().unwrap(),
        "export type GenericAutoBound<T> = { value: T, values: Array<T> }"
    );

    assert_eq!(
        export::<GenericAutoBound2::<()>>().unwrap(),
        "export type GenericAutoBound2<T> = { value: T, values: Array<T> }"
    );

    assert_eq!(
        export::<Container>().unwrap(),
        "export type Container = { foo: Generic<number>, bar: Array<Generic<number>>, baz: Record<string, Generic<string>> }"
    );
}

#[test]
fn generic_enum() {
    #[derive(Type)]
    enum Generic<A, B, C> {
        A(A),
        B(B, B, B),
        C(Vec<C>),
        D(Vec<Vec<Vec<A>>>),
        E { a: A, b: B, c: C },
        X(Vec<i32>),
        Y(i32),
        Z(Vec<Vec<i32>>),
    }

    assert_eq!(
        export::<Generic::<(), (), ()>>().unwrap(),
        r#"export type Generic<A, B, C> = { A: A } | { B: [B, B, B] } | { C: Array<C> } | { D: Array<Array<Array<A>>> } | { E: { a: A, b: B, c: C } } | { X: Array<number> } | { Y: number } | { Z: Array<Array<number>> }"#
    )
}

#[test]
fn generic_newtype() {
    #[derive(Type)]
    struct NewType<T>(Vec<Vec<T>>);

    assert_eq!(
        export::<NewType::<()>>().unwrap(),
        r#"export type NewType<T> = Array<Array<T>>"#
    );
}

#[test]
fn generic_tuple() {
    #[derive(Type)]
    struct Tuple<T>(T, Vec<T>, Vec<Vec<T>>);

    assert_eq!(
        export::<Tuple::<()>>().unwrap(),
        r#"export type Tuple<T> = [T, Array<T>, Array<Array<T>>]"#
    );
}

#[test]
fn generic_struct() {
    #[derive(Type)]
    struct Struct<T> {
        a: T,
        b: (T, T),
        c: (T, (T, T)),
        d: [T; 3],
        e: [(T, T); 3],
        f: Vec<T>,
        g: Vec<Vec<T>>,
        h: Vec<[(T, T); 3]>,
    }

    assert_eq!(
        export::<Struct::<()>>().unwrap(),
        "export type Struct<T> = { a: T, b: [T, T], c: [T, [T, T]], d: Array<T>, e: Array<[T, T]>, f: Array<T>, g: Array<Array<T>>, h: Array<Array<[T, T]>> }"
    )
}

// not currently possible in ts-rs hehe
#[test]
fn inline() {
    #[derive(Type)]
    struct Generic<T> {
        t: T,
    }

    #[derive(Type)]
    struct Container {
        g: Generic<String>,
        #[specta(inline)]
        gi: Generic<String>,
        #[specta(flatten)]
        t: Generic<String>,
    }

    assert_eq!(
        export::<Generic::<()>>().unwrap(),
        "export type Generic<T> = { t: T }"
    );
    assert_eq!(
        export::<Container>().unwrap(),
        "export type Container = ({ t: string }) & { g: Generic<string>, gi: { t: string } }"
    );
}

// #[test]
// fn default() {
//     #[derive(Type)]
//     struct A<T = String> {
//         t: T,
//     }
//     assert_eq!(
//         ts_export::<A::<()>>().unwrap(),
//         "export interface A<T = string> { t: T, }"
//     );

//     #[derive(Type)]
//     struct B<U = Option<A<i32>>> {
//         u: U,
//     }
//     assert_eq!(
//         ts_export::<B::<()>>().unwrap(),
//         "export interface B<U = A<number> | null> { u: U, }"
//     );

//     #[derive(Type)]
//     struct Y {
//         a1: A,
//         a2: A<i32>,
// https://github.com/Aleph-Alpha/ts-rs/issues/56
// TODO: fixme
// #[ts(inline)]
// xi: X,
// #[ts(inline)]
// xi2: X<i32>
// }
// assert_eq!(
//     ts_export::<Y>().unwrap(),
//     "interface Y { a1: A, a2: A<number> }"
// )
// }

// TODO

// #[test]
// fn trait_bounds() {
//     #[derive(Type)]
//     // TODO
//     struct A<T: ToString = i32> {
//         t: T,
//     }
//     assert_eq!(
//         ts_export::<A::<i32>>().unwrap(),
//         "export interface A<T = number> { t: T, }"
//     );

//     #[derive(Type)]
//     struct B<T: ToString + Debug + Clone + 'static>(T);
//     assert_eq!(
//         ts_export::<B::<&'static str>>().unwrap(),
//         "export type B<T> = T;"
//     );

//     #[derive(Type)]
//     enum C<T: Copy + Clone + PartialEq, K: Copy + PartialOrd = i32> {
//         A { t: T },
//         B(T),
//         C,
//         D(T, K),
//     }
//     assert_eq!(
//         ts_export::<C::<&'static str, i32>>().unwrap(),
//         "export type C<T, K = number> = { A: { t: T, } } | { B: T } | \"C\" | { D: [T, K] };"
//     );

//     #[derive(Type)]
//     struct D<T: ToString, const N: usize> {
//         t: [T; N],
//     }

//     assert_eq!(
//         ts_export::<D::<&str, 41>>().unwrap(),
//         "export interface D<T> { t: Array<T>, }"
//     )
// }
