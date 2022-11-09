use tauri::{AppHandle, Runtime, State, Window};

use crate::{DataType, DefOpts, TypedCommandArg};

pub enum TypedCommandArgWindowMarker {}
impl<R: Runtime> TypedCommandArg<TypedCommandArgWindowMarker> for Window<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

pub enum TypedCommandArgStateMarker {}
impl<'r, T: Send + Sync + 'static> TypedCommandArg<TypedCommandArgStateMarker> for State<'r, T> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}

pub enum TypedCommandArgAppHandleMarker {}
impl<R: Runtime> TypedCommandArg<TypedCommandArgAppHandleMarker> for AppHandle<R> {
    fn to_datatype(_: DefOpts) -> Option<DataType> {
        None
    }
}
