use std::{error, fmt};

use serde::Serialize;
use specta::Type;

#[derive(Serialize, Type)]
pub enum InternalError {
    FromValue(()),
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::FromValue(()) => write!(f, "Failed to convert value to input"),
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for InternalError {}
