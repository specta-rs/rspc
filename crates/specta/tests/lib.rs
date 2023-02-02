#![allow(unused_variables, dead_code)]

mod bigints;
mod datatype;
mod duplicate_ty_name;
mod macro_decls;
pub mod ts;
mod ts_rs;
mod ty_override;

#[test]
fn test_compile_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/macro/compile_error.rs");
}
