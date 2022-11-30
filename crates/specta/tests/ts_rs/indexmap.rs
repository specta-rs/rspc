#![cfg(feature = "indexmap")]

use indexmap::{IndexMap, IndexSet};
use specta::{ts, Type};

#[test]
fn indexmap() {
    #[derive(Type)]
    #[allow(dead_code)]
    struct Indexes {
        map: IndexMap<String, String>,
        set: IndexSet<String>,
    }

    assert_eq!(
        ts::inline::<Indexes>(),
        "{ map: Record<string, string>, set: Array<string> }"
    )
}
