use futures::{stream, Stream, StreamExt};
use serde_json::Value;

use crate::error::ExecError;

// TODO: Normalise into a `type Stream` for better errors like we do for resolvers???

pub trait IntoMiddlewareResult<M> {
    type Stream: Stream<Item = Result<Value, ExecError>> + Send + 'static;

    fn into_result(self) -> Result<Self::Stream, ExecError>;
}

pub enum TODOTemporaryOnlyValidMarker {}
impl<S: Stream<Item = Value> + Send + 'static> IntoMiddlewareResult<TODOTemporaryOnlyValidMarker>
    for S
{
    type Stream = stream::Map<S, fn(Value) -> Result<Value, ExecError>>;

    fn into_result(self) -> Result<Self::Stream, ExecError> {
        Ok(self.map(Ok))
    }
}

// TODO: Should we allow any `impl Serialize`??? would make it too easy to get wrong!!!

// TODO: Re-enable all these once the marker issue is sorted out

// const _: () = {
//     pub enum Marker {}
//     impl IntoMiddlewareResult<Marker> for () {}
// };

// const _: () = {
//     pub enum Marker {}
//     impl IntoMiddlewareResult<Marker> for Value {}
// };

// const _: () = {
//     pub enum Marker {}
//     impl IntoMiddlewareResult<Marker> for Result<Value, ExecError> {}
// };

// const _: () = {
//     pub enum Marker {}
//     impl<S: Stream<Item = Value>> IntoMiddlewareResult<Marker> for S {}
// };

// const _: () = {
//     pub enum Marker {}
//     impl<S: Stream<Item = Result<Value, ExecError>>> IntoMiddlewareResult<Marker> for S {}
// };

// TODO: Result containing all the `impl Stream` types
