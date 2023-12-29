use futures::Stream;
use serde_json::Value;

use crate::error::ExecError;

pub trait IntoMiddlewareResult<M> {
    // fn into_result(self) -> Result<Value, ExecError> {
    //     todo!();
    // }
}

pub enum TODOTemporaryOnlyValidMarker {}
impl<S: Stream<Item = Value>> IntoMiddlewareResult<TODOTemporaryOnlyValidMarker> for S {}

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
