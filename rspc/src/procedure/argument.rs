use serde::Deserializer;

use super::Input;

pub trait Argument<'de>: Sized {
    type Value: Input;

    fn into_value(self) -> Self::Value;

    fn into_deserializer(self) -> Result<impl Deserializer<'de>, Self> {
        Err::<serde_value::Value, _>(self)
    }
}

impl<'de, D: Deserializer<'de>> Argument<'de> for D {
    type Value = ();

    fn into_value(self) -> Self::Value {
        unreachable!();
    }

    fn into_deserializer(self) -> Result<impl Deserializer<'de>, Self> {
        Ok(self)
    }
}
