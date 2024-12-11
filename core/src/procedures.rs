use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::Procedure;

pub struct Procedures<TCtx>(HashMap<Cow<'static, str>, Procedure<TCtx>>);

impl<TCtx> From<HashMap<Cow<'static, str>, Procedure<TCtx>>> for Procedures<TCtx> {
    fn from(procedures: HashMap<Cow<'static, str>, Procedure<TCtx>>) -> Self {
        Self(procedures.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl<TCtx> Clone for Procedures<TCtx> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<TCtx> Into<Procedures<TCtx>> for &Procedures<TCtx> {
    fn into(self) -> Procedures<TCtx> {
        self.clone()
    }
}

impl<TCtx> fmt::Debug for Procedures<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

impl<TCtx> IntoIterator for Procedures<TCtx> {
    type Item = (Cow<'static, str>, Procedure<TCtx>);
    type IntoIter = std::collections::hash_map::IntoIter<Cow<'static, str>, Procedure<TCtx>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<TCtx> FromIterator<(Cow<'static, str>, Procedure<TCtx>)> for Procedures<TCtx> {
    fn from_iter<I: IntoIterator<Item = (Cow<'static, str>, Procedure<TCtx>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<TCtx> Deref for Procedures<TCtx> {
    type Target = HashMap<Cow<'static, str>, Procedure<TCtx>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TCtx> DerefMut for Procedures<TCtx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
