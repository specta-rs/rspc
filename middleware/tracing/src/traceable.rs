use std::fmt;

pub trait Traceable<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

#[doc(hidden)]
pub enum DebugMarker {}
impl<T: fmt::Debug> Traceable<DebugMarker> for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

#[doc(hidden)]
pub enum StreamMarker {}
// `rspc::Stream: !Debug` so the marker will never overlap
impl<S> Traceable<StreamMarker> for rspc::modern::Stream<S>
where
    S: futures::Stream,
    S::Item: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!(); // TODO: Finish this
    }
}
