use std::collections::HashSet;

use specta::{Type, TypeDefs, ts_dependencies};

#[derive(Type)]
struct Primitives {
    int: i32,
    b: bool,
}

#[derive(Type)]
struct DependsOnPrimitives {
    p: Primitives,
}

#[derive(Type)]
struct DependsOnPrimitives2 {
    p: Primitives,
    d: DependsOnPrimitives
}

#[derive(Type)]
struct InlinesPrimitives {
    #[specta(inline)]
    p: Primitives 
}

#[derive(Type)]
struct InlinesDependsOnPrimitives {
    #[specta(inline)]
    d: DependsOnPrimitives,
}

#[test]
fn test_deps() {
    let def = Primitives::def(&mut TypeDefs::new());
    assert!(ts_dependencies(&def).is_empty());

    let def = DependsOnPrimitives::def(&mut TypeDefs::new());
    assert_eq!(ts_dependencies(&def), vec!["Primitives"].into_iter().collect());

    let def = DependsOnPrimitives2::def(&mut TypeDefs::new());
    assert_eq!(ts_dependencies(&def), vec!["Primitives", "DependsOnPrimitives"].into_iter().collect());

    let def = InlinesPrimitives::def(&mut TypeDefs::new());
    assert_eq!(ts_dependencies(&def), vec![].into_iter().collect());

    let def = InlinesDependsOnPrimitives::def(&mut TypeDefs::new());
    write_dbg(&def);
    assert_eq!(ts_dependencies(&def), vec!["Primitives"].into_iter().collect());
}

fn write_dbg(o: impl std::fmt::Debug) {
    std::fs::write("./dbg.rs", format!("{:#?}", o)).unwrap();
}
