use std::fmt::Display;

pub trait Key: Ord + Clone + Display {
    fn to_val(&self) -> &'static str;
}

impl Key for &'static str {
    fn to_val(&self) -> &'static str {
        self
    }
}
