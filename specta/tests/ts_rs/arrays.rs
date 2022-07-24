use specta::{Type, to_ts};

#[test]
fn free() {
    assert_eq!(to_ts::<[String; 10]>(), "Array<string>")
}

#[test]
fn interface() {
    #[derive(Type)]
    struct Interface {
        #[allow(dead_code)]
        a: [i32; 10],
    }

    assert_eq!(to_ts::<Interface>(), "{ a: Array<number>, }")
}

#[test]
fn newtype() {
    #[derive(Type)]
    struct Newtype(#[allow(dead_code)] [i32; 10]);

    assert_eq!(to_ts::<Newtype>(), "Array<number>")
}
