use std::{borrow::Cow, panic::Location};

use thiserror::Error;

use crate::CompiledRouter;

/// TODO
#[derive(Debug, PartialEq, Eq)]
pub struct BuildError {
    pub(crate) cause: BuildErrorCause,
    #[cfg(debug_assertions)]
    pub(crate) name: Cow<'static, str>,
    #[cfg(debug_assertions)]
    pub(crate) loc: &'static Location<'static>,
}

impl BuildError {
    /// DO NOT USE IT, it's for unit testing only and may change without a major version bump.
    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub fn expose(&self) -> (String, String) {
        (self.name.to_string(), self.cause.to_string())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum BuildErrorCause {
    #[error(
        "a procedure or router name must be more than 1 character and less than 255 characters"
    )]
    InvalidName,
    #[error("a procedure or router name contains the character '{0}' which is not allowed. Names must be alphanumeric or have '_' or '-'")]
    InvalidCharInName(char),
    #[error(
        "a procedure or router name is using the name '{0}' which is reserved for internal use."
    )]
    ReservedName(String),
}

/// TODO
pub enum BuildResult<TCtx: 'static> {
    Ok(CompiledRouter<TCtx>),
    Err(Vec<BuildError>),
}

impl<TCtx: 'static> PartialEq for BuildResult<TCtx> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ok(_), Self::Ok(_)) => true,
            (Self::Err(e), Self::Err(e2)) => e == e2,
            _ => false,
        }
    }
}

impl<TCtx: 'static> BuildResult<TCtx> {
    pub fn unwrap(self) -> CompiledRouter<TCtx> {
        match self {
            Self::Ok(router) => router,
            Self::Err(errors) => {
                #[cfg(debug_assertions)]
                {
                    eprintln!("Error building rspc router\n");

                    for error in &errors {
                        eprintln!("Error at '{}' with procedure '{}':", error.loc, error.name);
                        eprintln!("\t error: {}\n", error.cause);
                    }
                }

                #[cfg(not(debug_assertions))]
                {
                    eprintln!("Error building rspc router. Please use a debug build for proper error reporting.\n");
                }

                std::process::exit(1);
            }
        }
    }
}
