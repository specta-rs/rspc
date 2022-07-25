use specta::{Type, ts_inline, ts_export};

#[test]
fn list() {
    #[derive(Type)]
    struct List {
        #[allow(dead_code)]
        data: Option<Vec<u32>>
    }

    assert_eq!(ts_inline::<List>(), "{ data: Array<number> | null }");
    assert_eq!(ts_export::<List>().unwrap(), "export interface List { data: Array<number> | null }");
}
