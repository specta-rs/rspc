use std::fmt;

use rspc::{Procedure, Router};
use rspc_procedure::ResolverError;
use serde::Serialize;
use specta::Type;

#[test]
fn errors() {
    let router = <Router>::new()
        .procedure(
            "abc",
            Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
        )
        .procedure(
            "abc",
            Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
        );

    assert_eq!(
        format!("{:?}", router.build().unwrap_err()),
        "[Duplicate procedure at path [\"abc\"]. Original: rspc/tests/router.rs:13:13 Duplicate: rspc/tests/router.rs:15:10\n]"
    );

    let router = <Router>::new()
        .procedure(
            "abc",
            Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
        )
        .merge(<Router>::new().procedure(
            "abc",
            Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
        ));

    assert_eq!(format!("{:?}", router.build().unwrap_err()), "[Duplicate procedure at path [\"abc\"]. Original: rspc/tests/router.rs:32:13 Duplicate: rspc/tests/router.rs:28:13\n]");

    let router = <Router>::new()
        .nest(
            "abc",
            <Router>::new().procedure(
                "kjl",
                Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
            ),
        )
        .nest(
            "abc",
            <Router>::new().procedure(
                "def",
                Procedure::builder().query(|_, _: ()| async { Ok::<_, Infallible>(()) }),
            ),
        );

    assert_eq!(format!("{:?}", router.build().unwrap_err()), "[Duplicate procedure at path [\"abc\"]. Original: rspc/tests/router.rs:42:17 Duplicate: rspc/tests/router.rs:45:10\n]");
}

#[derive(Type, Debug)]
pub enum Infallible {}

impl fmt::Display for Infallible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Serialize for Infallible {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unreachable!()
    }
}

impl std::error::Error for Infallible {}

impl rspc::Error for Infallible {
    fn into_resolver_error(self) -> ResolverError {
        unreachable!()
    }
}
