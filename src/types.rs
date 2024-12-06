use std::{borrow::Cow, collections::BTreeMap};

use specta::TypeCollection;

use crate::procedure::ProcedureType;

pub struct Types {
    pub(crate) types: TypeCollection,
    pub(crate) procedures: BTreeMap<Vec<Cow<'static, str>>, ProcedureType>,
}

// TODO: Traits

impl Types {
    // TODO: Expose inners for manual exporting logic
}
