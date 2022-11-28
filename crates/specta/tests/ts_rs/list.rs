use specta::{ts, Type};

#[test]
fn list() {
    #[derive(Type)]
    struct List {
        #[allow(dead_code)]
        data: Option<Vec<u32>>,
    }

    assert_eq!(
        ts::export::<List>().unwrap(),
        "export type List = { data: Array<number> | null }"
    );
}
