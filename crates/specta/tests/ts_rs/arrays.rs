use specta::{ts::ts_inline, Type};

#[test]
fn free() {
    assert_eq!(ts_inline::<[String; 10]>(), "Array<string>")
}

#[test]
fn interface() {
    #[derive(Type)]
    struct Interface {
        #[allow(dead_code)]
        a: [i32; 10],
    }

    assert_eq!(ts_inline::<Interface>(), "{ a: Array<number> }")
}

#[test]
fn newtype() {
    #[derive(Type)]
    struct Newtype(#[allow(dead_code)] [i32; 10]);

    assert_eq!(ts_inline::<Newtype>(), "Array<number>")
}
