use std::{borrow::Cow, collections::HashMap, fmt};

use crate::{procedure::Procedure, State};

pub struct Router<TCtx = ()>(HashMap<Cow<'static, str>, Procedure<TCtx>>);

impl<TCtx> fmt::Debug for Router<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Router").field(&self.0).finish()
    }
}

impl<TCtx> Default for Router<TCtx> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<TCtx> Router<TCtx> {
    pub fn procedure(
        mut self,
        name: impl Into<Cow<'static, str>>,
        procedure: Procedure<TCtx>,
    ) -> Self {
        let name = name.into();

        // TODO: Delayed: -> Running the procedure's `init` function with the plugin store (once all merged together).

        self.0.insert(name, procedure);

        self
    }

    pub fn state(&self) -> State {
        todo!();
    }

    pub fn merge(self, other: Self) -> Self {
        todo!();
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Cow<'static, str>, &Procedure<TCtx>)> {
        self.0.iter()
    }

    // TODO: Maybe remove this?
    pub fn get<'a>(&self, k: &str) -> Option<&Procedure<TCtx>> {
        self.0.get(k)
    }
}

impl<TCtx> FromIterator<(Cow<'static, str>, Procedure<TCtx>)> for Router<TCtx> {
    fn from_iter<I: IntoIterator<Item = (Cow<'static, str>, Procedure<TCtx>)>>(iter: I) -> Self {
        let mut router = Self::default();
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
