use std::path::PathBuf;

/// TODO
pub struct Config {
    pub(crate) export_bindings_on_build: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            export_bindings_on_build: None,
        }
    }

    /// export_bindings will export the bindings of the generated router to a folder every time the router is built.
    /// Note: The bindings are only exported when `debug_assertions` are enabled (Rust is in debug mode).
    pub fn export_ts_bindings<TPath>(mut self, export_path: TPath) -> Self
    where
        PathBuf: From<TPath>,
    {
        self.export_bindings_on_build = Some(PathBuf::from(export_path));
        self
    }
}
