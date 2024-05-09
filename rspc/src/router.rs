use std::{borrow::Cow, collections::HashMap};

use crate::procedure::Procedure;

#[derive(Debug)]
pub struct Router<TCtx = ()>(HashMap<Cow<'static, str>, Procedure<TCtx>>);

impl<TCtx> Default for Router<TCtx> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<TCtx> Router<TCtx> {
    pub fn procedure(mut self, procedure: Procedure<TCtx>) -> Self {
        todo!();
    }

    pub fn merge(self, other: Self) -> Self {
        todo!();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Procedure<TCtx>)> {
        self.0.iter()
    }
}

impl<TCtx> FromIterator<(Cow<'static, str>, Procedure<TCtx>)> for Router<TCtx> {
    fn from_iter<I: IntoIterator<Item = (Cow<'static, str>, Procedure<TCtx>)>>(iter: I) -> Self {
        let mut router = Router::<TCtx>::default();
        for (path, procedure) in iter {
            router.0.insert(path, procedure);
        }
        router
    }
}

impl<TCtx> IntoIterator for Router<TCtx> {
    type Item = (Cow<'static, str>, Procedure<TCtx>);
    // TODO: This leaks the `HashMap` implementation detail into the public API. It would be nice if Rust let us `type IntoIter = impl Iterator<Item = ...>;`.
    type IntoIter = std::collections::hash_map::IntoIter<Cow<'static, str>, Procedure<TCtx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
