use std::any::{Any, TypeId};

use serde::{de::DeserializeOwned, Deserialize};

use super::Input;

/// This is the internal version of `Input`.
///
/// It's sealed because:
///  - `erased_serde` should not appear in the public API.
///  - It's methods are relatively unsafe due to the usage of `Option` to ensure dyn-safety with owned values.
pub(super) trait InputSealed: 'static {
    // This method returns `Option<T>` as `dyn Any` so we can take the value out of the `Option` while remaining dyn-safe.
    fn to_option_dyn_any(&mut self) -> &mut dyn Any;

    fn to_value(&mut self) -> Option<Result<serde_value::Value, serde_value::SerializerError>>;
}
impl<T: Input> InputSealed for Option<T> {
    fn to_option_dyn_any(&mut self) -> &mut dyn Any {
        self
    }

    fn to_value(&mut self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        // let t = self.take().expect("value already taken");
        // let t = T::deserialize(t).unwrap(); // TODO: How to handle this one???

        // let mut format = <dyn erased_serde::Deserializer>::erase(serde_value::Value::Unit);

        // let y: serde_value::Value = erased_serde::deserialize(&mut format).unwrap();

        // <serde_value::Value::Unit as Deserialize>::deserialize().unwrap();

        // let value =
        // let y =
        //     <<T as Input>::Input as Deserialize>::deserialize(serde_value::Value::Unit).unwrap();

        // let value = serde_value::ValueDeserializer::new(value)

        // Some(serde_value::to_value(&t))

        todo!();
    }
}

pub struct InputValue<'a> {
    pub(super) type_id: TypeId,
    // This holds `Option<T>` so we can `.take()` an owned `T` while remaining dyn-safe.
    pub(super) inner: &'a mut dyn Any,
}

impl<'a> InputValue<'a> {
    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn downcast<T: Any + 'static>(self) -> Option<T> {
        Some(
            self.inner
                .downcast_mut::<Option<T>>()?
                .take()
                .expect("value already taken"),
        )
    }

    pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, ()> {
        todo!();
    }
}
