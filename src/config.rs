use std::path::PathBuf;

/// TODO
#[derive(Default)]
pub struct Config {
    pub(crate) export_bindings_on_build: Option<PathBuf>,
    pub(crate) bindings_header: Option<&'static str>,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    /// will export the bindings of the generated router to a folder every time the router is built.
    /// Note: The bindings are only exported when `debug_assertions` are enabled (Rust is in debug mode).
    pub fn export_ts_bindings<TPath>(mut self, export_path: TPath) -> Self
    where
        PathBuf: From<TPath>,
    {
        self.export_bindings_on_build = Some(PathBuf::from(export_path));
        self
    }

    /// allows you to add a custom string to the top of the exported Typescript bindings file.
    /// This is useful if you want to disable ESLint or Prettier.
    pub fn set_ts_bindings_header(mut self, custom: &'static str) -> Self {
        self.bindings_header = Some(custom);
        self
    }
}
