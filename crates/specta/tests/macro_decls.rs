use specta::Type;

use crate::ts::assert_ts;

macro_rules! field_ty_macro {
    () => {
        String
    };
}

#[derive(Type)]
pub struct MacroStruct(field_ty_macro!());

#[derive(Type)]
pub struct MacroStruct2 {
    demo: field_ty_macro!(),
}

#[derive(Type)]
pub enum MacroEnum {
    Demo(field_ty_macro!()),
    Demo2 { demo2: field_ty_macro!() },
}

#[test]
fn test_macro_in_decls() {
    assert_ts!(MacroStruct, "string");
    assert_ts!(MacroStruct2, "{ demo: string }");
    assert_ts!(MacroEnum, "{ Demo: string } | { Demo2: { demo2: string } }");
}
