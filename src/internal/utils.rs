use pin_project::pin_project;

#[pin_project(project = _PinnedOptionProj)]
pub(crate) enum PinnedOption<T> {
    Some(#[pin] T),
    None,
}

pub(crate) use _PinnedOptionProj as PinnedOptionProj;
