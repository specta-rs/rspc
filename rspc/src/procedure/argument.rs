use serde::Deserializer;

use super::Input;

/// An argument to [`Procedure::exec`](crate::procedure::Procedure::exec) which is deserialized or downcasted to the [`Input`] of the procedure.
///
/// This trait has a built in implementation for any [`Deserializer`](serde::Deserializer)'s so you can provide:
///  - [`serde_json::Value`]
///  - [`serde_value::Value`]
///  - [`serde_json::Deserializer::from_str`]
///  - etc.
///
/// ## How this works?
///
/// If you provide a type which implements [`Deserialize`](serde::Deserialize) we will use it to construct the [`Input`] value of the procedure, otherwise downcasting will be used.
///
/// ## Implementation for custom types
///
/// Say you have a type `MyCoolThing` which you want to use as an argument to an rspc procedure:
///
/// ```
/// pub struct MyCoolThing(pub String);
///
/// impl<'de> Argument<'de> for MyCoolThing {
///     type Value = Self;
///     
///     fn into_value(self) -> Self::Value {
///         self
///     }
/// }
///
/// impl Input for MyCoolThing {
///     fn from_value(value: ProcedureInput<Self>) -> Result<Self, ()> {
///        Ok(value.downcast().ok_or(())?)
///     }
/// }
/// ```
pub trait Argument<'de>: Sized {
    /// The value which is available from your [`Input`] implementation to downcast from.
    ///
    /// This exists so your able to accept `SomeType<T>` as an [`Argument`], but then type-erase to `SomeType<Box<dyn Trait>>` so your [`Input`] implementation is able to downcast the value.
    ///
    /// It's recommended to set this to `Self` unless you hit the case described above.
    type Value: Input;

    /// Convert self into `Self::Value`
    fn into_value(self) -> Self::Value;

    /// Convert self into a [`Deserializer`](serde::Deserializer) if possible or return the original value.
    ///
    /// This will be executed and if it returns `Err(Self)` we will fallback to [`Self::into_value`] and use downcasting.
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
