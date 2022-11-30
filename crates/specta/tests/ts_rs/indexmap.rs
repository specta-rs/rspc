#![cfg(feature = "indexmap")]

use indexmap::{IndexMap, IndexSet};
use specta::{ts::ts_export, Type};

#[test]
fn indexmap() {
    #[derive(Type)]
    #[allow(dead_code)]
    struct Indexes {
        map: IndexMap<String, String>,
        set: IndexSet<String>,
    }

    assert_eq!(
        ts_export::<Indexes>().unwrap(),
        "export type Indexes = { map: Record<string, string>, set: Array<string> }"
    )
}
