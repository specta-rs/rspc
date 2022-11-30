use specta::{
    datatype::{DataType, LiteralType},
    ts, DataTypeFrom,
};

#[derive(DataTypeFrom)]
pub struct MyEnum(pub Vec<DataType>);
fn main() {
    let e = MyEnum(vec![
        DataType::Literal(LiteralType::String("A".to_string())),
        DataType::Literal(LiteralType::String("B".to_string())),
    ]);

    assert_eq!(
        ts::export_datatype(&e.into()).unwrap(),
        "export type MyEnum = \"A\" | \"B\""
    );
}
