use specta::*;

#[derive(Type)]
struct Bruh<A, B> {
    a: A,
    b: B,
}

#[test]
fn test() {
    std::fs::write("./lmao.swift", swift::export::<Bruh<i32, String>>());
}
