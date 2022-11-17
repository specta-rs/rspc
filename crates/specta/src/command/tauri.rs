use tauri::{AppHandle, Runtime, State, Window};

use crate::{command::TypedCommandArg, DataType, DefOpts};

#[doc(hidden)]
pub enum TypedCommandArgWindowMarker {}
impl<R: Runtime> TypedCommandArg<TypedCommandArgWindowMarker> for Window<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

#[doc(hidden)]
pub enum TypedCommandArgStateMarker {}
impl<'r, T: Send + Sync + 'static> TypedCommandArg<TypedCommandArgStateMarker> for State<'r, T> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

#[doc(hidden)]
pub enum TypedCommandArgAppHandleMarker {}
impl<R: Runtime> TypedCommandArg<TypedCommandArgAppHandleMarker> for AppHandle<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}
