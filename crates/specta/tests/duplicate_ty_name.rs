// TODO: Unit test multiple different types with the same name. Show throw runtime error.

// TODO: This can't be done and support `--all-features` without the ability to exclude a type from being exported using the `export` flag. Make that then implement this test!

pub struct One {
    pub name: String,
}

pub struct Two {
    pub name: String,
}

pub struct Three {
    pub name: String,
}

#[ignore] // TODO: Remove once working
#[test]
fn test_duplicate_ty_name() {}
