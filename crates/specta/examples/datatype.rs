use specta::{
    datatype::{DataType, LiteralType},
    ts::ts_export_datatype,
    ToDataType,
};

#[derive(ToDataType)]
pub struct MyEnum(pub Vec<DataType>);
fn main() {
    let e = MyEnum(vec![
        DataType::Literal(LiteralType::String("A".to_string())),
        DataType::Literal(LiteralType::String("B".to_string())),
    ]);

    assert_eq!(
        ts_export_datatype(&e.to_data_type()).unwrap(),
        "export type MyEnum = \"A\" | \"B\""
    );
}
