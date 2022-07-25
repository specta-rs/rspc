use specta::{Type, ts_inline};

#[derive(Type)]
struct Unit;

#[derive(Type)]
struct Unit2 {}

#[derive(Type)]
struct Unit3();

#[test]
fn test() {
    assert_eq!("null", ts_inline::<Unit>());
    assert_eq!("null", ts_inline::<Unit2>());
    assert_eq!("null", ts_inline::<Unit3>());
}
