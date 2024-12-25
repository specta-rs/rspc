//! TODO: A temporary module to allow for interop between modern and legacy code.

// TODO: Remove this once we remove the legacy executor.
#[doc(hidden)]
#[derive(Clone)]
pub struct LegacyErrorInterop(pub String);
impl std::fmt::Debug for LegacyErrorInterop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LegacyErrorInterop({})", self.0)
    }
}
impl std::fmt::Display for LegacyErrorInterop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LegacyErrorInterop({})", self.0)
    }
}
impl std::error::Error for LegacyErrorInterop {}
