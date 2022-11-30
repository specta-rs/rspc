use specta::{
    ts::{export, inline},
    Type,
};

#[test]
fn test_tuple() {
    type Tuple = (String, i32, (i32, i32));
    assert_eq!("[string, number, [number, number]]", inline::<Tuple>());
}

#[test]
fn test_newtype() {
    #[derive(Type)]
    struct NewType(String);

    assert_eq!("export type NewType = string", export::<NewType>().unwrap());
}

#[test]
fn test_tuple_newtype() {
    #[derive(Type)]
    struct TupleNewType(String, i32, (i32, i32));
    assert_eq!(
        "export type TupleNewType = [string, number, [number, number]]",
        export::<TupleNewType>().unwrap()
    )
}
