use std::fmt;

use serde::Serialize;
use specta::Type;

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

impl crate::modern::Error for Infallible {
    fn status(&self) -> u16 {
        unreachable!()
    }
}
