//! These are the internals of create-rspc-app
//!
//! ## Warning
//!
//! These APIs are meant for internal use so you are using them at your own risk.
//! Expect APIs to break without a prior notice and without semver.

pub mod database {
    pub use crate::database::*;
}

pub mod framework {
    pub use crate::framework::*;
}

pub mod frontend_framework {
    pub use crate::frontend_framework::*;
}

pub mod generator {
    pub use crate::generator::*;
}
