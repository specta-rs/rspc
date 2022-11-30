use tauri::{AppHandle, Runtime, State, Window};

use crate::{function::SpectaFunctionArg, DataType, DefOpts};

#[doc(hidden)]
pub enum TauriMarker {}

impl<R: Runtime> SpectaFunctionArg<TauriMarker> for Window<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

impl<'r, T: Send + Sync + 'static> SpectaFunctionArg<TauriMarker> for State<'r, T> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

impl<R: Runtime> SpectaFunctionArg<TauriMarker> for AppHandle<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}
