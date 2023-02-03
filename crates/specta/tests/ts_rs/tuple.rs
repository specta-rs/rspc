use specta::Type;

use crate::ts::assert_ts;

#[test]
fn test_tuple() {
    type Tuple = (String, i32, (i32, i32));
    assert_ts!(Tuple, "[string, number, [number, number]]");
}

#[test]
fn test_newtype() {
    #[derive(Type)]
    struct NewType(String);

    assert_ts!(NewType, "string");
}

#[test]
fn test_tuple_newtype() {
    #[derive(Type)]
    struct TupleNewType(String, i32, (i32, i32));
    assert_ts!(TupleNewType, "[string, number, [number, number]]")
}
