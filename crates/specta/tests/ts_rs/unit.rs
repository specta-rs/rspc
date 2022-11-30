use specta::{ts::inline, Type};

#[derive(Type)]
struct Unit;

#[derive(Type)]
struct Unit2 {}

#[derive(Type)]
struct Unit3();

#[test]
fn test() {
    assert_eq!("null", inline::<Unit>());
    assert_eq!("null", inline::<Unit2>());
    assert_eq!("null", inline::<Unit3>());
}
