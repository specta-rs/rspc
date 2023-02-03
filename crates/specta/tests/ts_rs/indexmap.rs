#![cfg(feature = "indexmap")]

use indexmap::{IndexMap, IndexSet};
use specta::Type;

use crate::ts::assert_ts;

#[test]
fn indexmap() {
    #[derive(Type)]
    #[allow(dead_code)]
    struct Indexes {
        map: IndexMap<String, String>,
        set: IndexSet<String>,
    }

    assert_ts!(Indexes, "{ map: { [key: string]: string }; set: string[] }");
}
