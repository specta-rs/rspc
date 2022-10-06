use assert_json_diff::assert_json_eq;
use normi::Object;
use serde_json::json;
use std::cell::RefCell;
use trybuild::TestCases;

// #[test]
// fn macro_expected_failures() {
//     let t = TestCases::new();
//     t.compile_fail("tests/macros/fail.rs");
// }

#[test]
fn test_rename() {
    #[derive(Object)]
    #[normi(rename = "NewStructName")]
    struct Struct {
        #[normi(id)]
        id: String,
    }

    let v = Struct {
        id: "hello".to_string(),
    };

    assert_eq!(<Struct as Object>::type_name(), "NewStructName");
    assert_eq!(v.id().unwrap(), json!("hello"));
    assert_json_eq!(
        serde_json::to_value(&v.normalize().unwrap()).unwrap(),
        json!({
            "__id": "hello",
            "__type": "NewStructName",
            "id": "hello"
        })
    );
}

// TODO: It would be nice to support this in the future but it's not worth it for now
// #[test]
// fn test_unit_struct() {
//     #[derive(Object)]
//     struct Struct {
//         #[normi(id)]
//         id: String,
//     }

//     #[derive(Object)]
//     struct UnitRef(#[normi(flatten)] Struct);

//     let v = UnitRef(Struct {
//         id: "hello".to_string(),
//     });

//     assert_eq!(<UnitRef as Object>::type_name(), "UnitRef");
//     assert_eq!(v.id().unwrap(), json!("hello"));
//     assert_json_eq!(
//         serde_json::to_value(&v.normalize().unwrap()).unwrap(),
//         json!({"__id": "hello", "__type": "UnitRef", "id": "hello"})
//     );
// }

#[test]
fn test_simple_struct() {
    #[derive(Object)]
    struct Struct {
        #[normi(id)]
        id: i32,
        b: String,
        c: (i32, String, RefCell<i32>),
        d: Vec<String>,
        e: Option<String>,
    }

    let v = Struct {
        id: 5,
        b: "Hello".into(),
        c: (5, "Bruh".into(), RefCell::new(42)),
        d: [].to_vec(),
        e: None,
    };

    assert_eq!(<Struct as Object>::type_name(), "Struct");
    assert_eq!(v.id().unwrap(), json!(5));
    assert_json_eq!(
        serde_json::to_value(&v.normalize().unwrap()).unwrap(),
        json!({
            "__id": 5,
            "__type": "Struct",
            "b": "Hello",
            "c": [5, "Bruh", 42],
            "d": [],
            "e": null,
            "id": 5
        })
    );
}

#[test]
fn test_advanced_struct() {
    #[derive(Object)]
    struct User {
        #[normi(id)]
        id: String,
        name: String,
    }

    #[derive(Object)]
    struct Struct {
        #[normi(id)]
        id: i32,
        #[normi(refr)]
        users: Vec<User>,
        #[normi(refr)]
        owner: User,
    }

    let v = Struct {
        id: 42,
        users: [
            User {
                id: "a".into(),
                name: "Monty".into(),
            },
            User {
                id: "b".into(),
                name: "Millie".into(),
            },
        ]
        .into_iter()
        .collect::<Vec<_>>(),
        owner: User {
            id: "c".into(),
            name: "Oscar".into(),
        },
    };

    assert_eq!(<Struct as Object>::type_name(), "Struct");
    assert_eq!(v.id().unwrap(), json!(42));
    assert_json_eq!(
        serde_json::to_value(&v.normalize().unwrap()).unwrap(),
        json!({
            "__id": 42,
            "__type": "Struct",
            "id": 42,
            "users": {
                "__type": "User",
                "edges": [
                    { "__id": "a", "__type": "User", "id": "a", "name": "Monty"},
                    { "__id": "b", "__type": "User", "id": "b", "name": "Millie" }
                ],
            },
            "owner": {
                "__id": "c",
                "__type": "User",
                "id": "c",
                "name": "Oscar"
            }
        })
    );
}

#[test]
fn test_tuple_struct() {
    #[derive(Object)]
    struct TupleStruct(#[normi(id)] String, i32);

    let v = TupleStruct("one".into(), 42);

    assert_eq!(<TupleStruct as Object>::type_name(), "Unit");
    assert_eq!(v.id().unwrap(), json!("one"));
    assert_json_eq!(
        serde_json::to_value(&v.normalize().unwrap()).unwrap(),
        json!({
            "__id": "one",
            "__type": "Unit",
            "data": ["one", 42]
        })
    );
}

#[test]
fn test_tuple_struct_refr() {
    #[derive(Object)]
    struct Struct {
        #[normi(id)]
        id: i32,
    }

    #[derive(Object)]
    struct TupleStruct(#[normi(id)] String, i32, #[normi(refr)] Struct);

    assert_eq!(<TupleStruct as Object>::type_name(), "Unit");
    assert_eq!(v.id().unwrap(), json!("one"));
    assert_json_eq!(
        serde_json::to_value(&v.normalize().unwrap()).unwrap(),
        json!({
            "__id": "one",
            "__type": "Unit",
            "data": ["one", 42]
        })
    );
}

#[test]
fn test_enum() {
    // #[derive(Object)]
    // enum TestEnum {
    //     Unit,
    //     Single(i32),
    //     Multiple(i32, i32),
    //     Struct { a: i32 },
    // }
}

#[test]
fn test_externally_tagged_enum() {
    // #[derive(Object)]
    // enum AorB {
    //     A(Struct),
    //     B(Struct),
    // }
}

#[test]
fn test_internally_tagged_enum() {
    // #[derive(Object)]
    // #[serde(tag = "type")]
    // enum AorB {
    //     A(Struct),
    //     B(Struct),
    // }
}

#[test]
fn test_adjacently_tagged_enum() {
    // #[derive(Object)]
    // #[serde(tag = "t", content = "c")]
    // enum AorB {
    //     A(Struct),
    //     B(Struct),
    // }
}

#[test]
fn test_untagged_enum() {
    // #[derive(Object)]
    // #[serde(untagged)]
    // enum AorB {
    //     A(Struct),
    //     B(Struct),
    // }
}

#[test]
fn test_generic_struct() {
    // #[derive(Object)]
    // struct GenericStruct<T> {
    //     #[normi(id)]
    //     id: String,
    //     arg: T,
    // }
}

#[test]
fn test_generic_struct_refr() {
    // #[derive(Object)]
    // struct GenericStruct<T> {
    //     #[normi(id)]
    //     id: String,
    //     #[normi(refr)]
    //     arg: T,
    // }
}
