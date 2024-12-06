use std::{borrow::Cow, collections::BTreeMap};

use specta::TypeCollection;

use crate::procedure::ProcedureType;

#[derive(Clone)]
pub(crate) enum TypesOrType {
    Type(ProcedureType),
    Types(BTreeMap<Cow<'static, str>, TypesOrType>),
}

pub struct Types {
    pub(crate) types: TypeCollection,
    pub(crate) procedures: BTreeMap<Cow<'static, str>, TypesOrType>,
}

// TODO: Traits

impl Types {
    // TODO: Expose inners for manual exporting logic
}
