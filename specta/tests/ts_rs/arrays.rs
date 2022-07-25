use specta::{Type, ts_definition};

#[test]
fn free() {
    assert_eq!(ts_definition::<[String; 10]>(), "Array<string>")
}

#[test]
fn interface() {
    #[derive(Type)]
    struct Interface {
        #[allow(dead_code)]
        a: [i32; 10],
    }

    assert_eq!(ts_definition::<Interface>(), "{ a: Array<number> }")
}

#[test]
fn newtype() {
    #[derive(Type)]
    struct Newtype(#[allow(dead_code)] [i32; 10]);

    assert_eq!(ts_definition::<Newtype>(), "Array<number>")
}
