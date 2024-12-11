use std::{borrow::Cow, collections::BTreeMap, fmt};

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

impl fmt::Debug for Types {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Types").finish()
    }
}

// TODO: Traits

impl Types {
    // TODO: Expose inners for manual exporting logic
}
