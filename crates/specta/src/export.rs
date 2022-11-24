use crate::*;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref TYPES: Mutex<BTreeMap<&'static str, DataType>> = Mutex::new(Default::default());
}
