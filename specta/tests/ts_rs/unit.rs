use specta::{Type, ts_definition};

#[derive(Type)]
struct Unit;

#[derive(Type)]
struct Unit2 {}

#[derive(Type)]
struct Unit3();

#[test]
fn test() {
    assert_eq!("null", ts_definition::<Unit>());
    assert_eq!("null", ts_definition::<Unit2>());
    assert_eq!("null", ts_definition::<Unit3>());
}
