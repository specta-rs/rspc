use std::{cell::Cell, rc::Rc, sync::Arc};

use specta::{ts::inline, Type};

#[derive(Type)]
struct A {
    x1: Arc<i32>,
    y1: Cell<i32>,
}

#[derive(Type)]
struct B {
    a1: Box<A>,
    #[specta(inline)]
    a2: A,
}

#[derive(Type)]
struct C {
    b1: Rc<B>,
    #[specta(inline)]
    b2: B,
}

#[test]
fn test_nested() {
    assert_eq!(
        inline::<C>(),
        "{ b1: B, b2: { a1: A, a2: { x1: number, y1: number } } }"
    );
}
