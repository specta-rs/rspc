pin_project_lite::pin_project! {
    #[project = PinnedOptionProj]
    pub enum PinnedOption<T> {
        Some {
            #[pin]
            v: T,
        },
        None,
    }
}

impl<T> From<T> for PinnedOption<T> {
    fn from(value: T) -> Self {
        Self::Some { v: value }
    }
}
