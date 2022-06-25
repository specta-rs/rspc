pub trait KeyDefinition {
    type Key;
}

pub trait Key<TKey, TArg> {
    type Arg;

    fn to_val(&self) -> &'static str;
}

impl KeyDefinition for &'static str {
    type Key = &'static str;
}

impl<TArg> Key<&'static str, TArg> for &'static str {
    type Arg = TArg;

    fn to_val(&self) -> &'static str {
        self
    }
}
