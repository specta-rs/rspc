use specta::{ts::ts_export, Type};

#[test]
fn list() {
    #[derive(Type)]
    struct List {
        #[allow(dead_code)]
        data: Option<Vec<u32>>,
    }

    assert_eq!(
        ts_export::<List>().unwrap(),
        "export type List = { data: Array<number> | null }"
    );
}
