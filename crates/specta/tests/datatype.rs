use specta::{ts, DataType, DataTypeFrom, LiteralType};

use crate::ts::assert_ts;

#[derive(DataTypeFrom)]
struct Procedures1 {
    pub queries: Vec<DataType>,
}

// Testing using `DataTypeFrom` and `Type` together.
#[derive(DataTypeFrom, specta::Type)] // This derive bit gets passed into the macro
#[specta(rename = "ProceduresDef")]
struct Procedures2 {
    #[specta(type = String)] // This is a lie but just for the test
    pub queries: Vec<DataType>,
}

#[test]
fn test_datatype() {
    let dt: DataType = Procedures1 { queries: vec![] }.into();
    assert_eq!(
        &ts::datatype(&Default::default(), &dt).unwrap(),
        "{ queries: never }"
    );

    let dt: DataType = Procedures1 {
        queries: vec![
            DataType::Literal(LiteralType::String("A".to_string())),
            DataType::Literal(LiteralType::String("B".to_string())),
            DataType::Literal(LiteralType::bool(true)),
            DataType::Literal(LiteralType::i32(42)),
        ],
    }
    .into();
    assert_eq!(
        &ts::datatype(&Default::default(), &dt).unwrap(),
        r#"{ queries: "A" | "B" | true | 42 }"#
    );

    let dt: DataType = Procedures2 { queries: vec![] }.into();
    assert_eq!(
        &ts::datatype(&Default::default(), &dt).unwrap(),
        "{ queries: never }"
    );

    assert_ts!(Procedures2, "{ queries: string }");
}
