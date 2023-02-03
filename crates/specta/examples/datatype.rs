//! This file show how to use an advanced API of Specta.
//! You probably shouldn't be using this in application code but if your building a library on Specta it will be useful.

use specta::{
    datatype::{DataType, LiteralType},
    DataTypeFrom,
};

#[derive(DataTypeFrom)]
pub struct MyEnum(pub Vec<DataType>);

fn main() {
    let _e = MyEnum(vec![
        DataType::Literal(LiteralType::String("A".to_string())),
        DataType::Literal(LiteralType::String("B".to_string())),
    ]);

    // TODO: Fix this
    // let ts = ts::export_datatype(&ExportConfiguration::default(), &DataTypeExt {
    //     name: "",
    //     comments: "",
    //     sid: todo!(),
    //     impl_location: todo!(),
    //     inner: e.into(),
    // })).unwrap();

    // println!("{ts}");
    // assert_eq!(ts, "export type MyEnum = \"A\" | \"B\"");
}
