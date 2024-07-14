/// Return a [`Stream`](futures::Stream) of values from a [`Procedure::query`](procedure::ProcedureBuilder::query) or [`Procedure::mutation`](procedure::ProcedureBuilder::mutation).
///
/// ## Why not a subscription?
///
/// A [`subscription`](procedure::ProcedureBuilder::subscription) must return a [`Stream`](futures::Stream) so it would be fair to question when you would use this.
///
/// A [`query`](procedure::ProcedureBuilder::query) or [`mutation`](procedure::ProcedureBuilder::mutation) produce a single result where a subscription produces many discrete values.
///
/// Using [`rspc::Stream`](Self) within a query or mutation will result in your procedure returning a collection (Eg. `Vec`) of [`Stream::Item`](futures::Stream) on the frontend.
///
/// This means it would be well suited for streaming the result of a computation or database query while a subscription would be well suited for a chat room.
///
/// ## Usage
/// **WARNING**: This example shows the low-level procedure API. You should refer to [`Rspc`](crate::Rspc) for the high-level API.
/// ```rust
/// use futures::stream::once;
///
/// <Procedure>::builder().query(|_, _: ()| async move { rspc::Stream(once(async move { 42 })) });
/// ```
///
pub struct Stream<T>(pub T)
where
    T: futures::Stream,
    T::Item: AnyResult;

// TODO: Diagnostic if we keep this
pub trait AnyResult {}
impl<T, E> AnyResult for Result<T, E> {}
