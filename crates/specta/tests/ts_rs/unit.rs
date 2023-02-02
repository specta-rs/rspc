use specta::Type;

use crate::ts::assert_ts;

#[derive(Type)]
struct UnitA;

#[derive(Type)]
struct UnitB {}

#[derive(Type)]
struct UnitC();

#[test]
fn test() {
    assert_ts!(UnitA, "null");
    assert_ts!(UnitB, "null");
    assert_ts!(UnitC, "null");
}
