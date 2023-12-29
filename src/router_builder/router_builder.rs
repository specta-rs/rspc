use std::{borrow::Cow, panic::Location};

use crate::{
    procedure::{HasResolver, Procedure},
    router::Router,
    router_builder2::{
        edit_build_error_name, new_build_error, BuildError, BuildErrorCause, BuildResult,
    },
};

pub(crate) type ProcedureBuildFn<TCtx> = Box<dyn FnOnce(Cow<'static, str>, &mut Router<TCtx>)>;

pub struct RouterBuilder<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    procedures: Vec<(Cow<'static, str>, ProcedureBuildFn<TCtx>)>,
    errors: Vec<BuildError>,
}

impl<TCtx> RouterBuilder<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    /// Constructs a new `Router`.
    /// Avoid using this directly, use [`Rspc::router`] instead so the types can be inferred.
    pub(crate) fn _internal_new() -> Self {
        Self {
            procedures: Vec::new(),
            errors: Vec::new(),
        }
    }

    #[track_caller]
    pub fn procedure(mut self, key: &'static str, procedure: Procedure<HasResolver<TCtx>>) -> Self {
        if let Some(cause) = is_valid_name(key) {
            self.errors.push(new_build_error(
                cause,
                #[cfg(debug_assertions)]
                Cow::Borrowed(key),
                #[cfg(debug_assertions)]
                Location::caller(),
            ));
        }

        self.procedures.push((Cow::Borrowed(key), procedure.take()));
        self
    }

    #[track_caller]
    #[allow(unused_mut)]
    pub fn merge(mut self, prefix: &'static str, mut r: RouterBuilder<TCtx>) -> Self {
        if let Some(cause) = is_valid_name(prefix) {
            self.errors.push(new_build_error(
                cause,
                #[cfg(debug_assertions)]
                Cow::Borrowed(prefix),
                #[cfg(debug_assertions)]
                Location::caller(),
            ));
        }

        #[cfg(not(debug_assertions))]
        {
            self.errors.append(&mut r.errors);
        }

        #[cfg(debug_assertions)]
        {
            self.errors.extend(&mut r.errors.into_iter().map(|mut err| {
                edit_build_error_name(&mut err, |name| Cow::Owned(format!("{}.{}", prefix, name)));
                err
            }));
        }

        self.procedures.extend(
            r.procedures
                .into_iter()
                .map(|(name, p)| (Cow::Owned(format!("{}.{}", prefix, name)), p)),
        );

        self
    }

    pub fn build(self) -> BuildResult<TCtx> {
        if !self.errors.is_empty() {
            return BuildResult::Err(self.errors);
        }

        let mut router = Router::default();

        for (key, build_fn) in self.procedures.into_iter() {
            // TODO: Pass in the `key` here with the router merging prefixes already applied so it's the final runtime key
            (build_fn)(key, &mut router);
        }

        BuildResult::Ok(router)
    }
}

pub(crate) fn is_valid_name(name: &str) -> Option<BuildErrorCause> {
    if name.is_empty() || name.len() > 255 {
        return Some(BuildErrorCause::InvalidName);
    }

    for c in name.chars() {
        if !(c.is_alphanumeric() || c == '_' || c == '-' || c == '~') {
            return Some(BuildErrorCause::InvalidCharInName(c));
        }
    }

    if name == "rspc" || name == "_batch" {
        return Some(BuildErrorCause::ReservedName(name.to_string()));
    }

    None
}
