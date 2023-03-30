use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

pub trait MiddlewareArgMapper: Send + Sync {
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    type Output<T>: Serialize
    where
        T: Serialize;
    type State: Send + 'static;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State);
}

// TODO: Making this private or put a field on it so it can't be constructed out of the crate
pub struct MiddlewareArgMapperPassthrough;

impl MiddlewareArgMapper for MiddlewareArgMapperPassthrough {
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;
    type Output<T> = T where T: Serialize;
    type State = ();

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        (arg, ())
    }
}

// TODO: Remove this
impl MiddlewareArgMapper for () {
    type State = ();
    type Output<T> = T where T: Serialize;
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (Self::Output<T>, Self::State) {
        (arg, ())
    }
}
