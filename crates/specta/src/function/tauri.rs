use tauri::{AppHandle, Runtime, State, Window};

use crate::{function::SpectaFunctionArg, DataType, DefOpts};

#[doc(hidden)]
pub enum SpectaFunctionArgWindowMarker {}
impl<R: Runtime> SpectaFunctionArg<SpectaFunctionArgWindowMarker> for Window<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

#[doc(hidden)]
pub enum SpectaFunctionArgStateMarker {}
impl<'r, T: Send + Sync + 'static> SpectaFunctionArg<SpectaFunctionArgStateMarker>
    for State<'r, T>
{
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

#[doc(hidden)]
pub enum SpectaFunctionArgAppHandleMarker {}
impl<R: Runtime> SpectaFunctionArg<SpectaFunctionArgAppHandleMarker> for AppHandle<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}
