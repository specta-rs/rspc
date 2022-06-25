pub trait KeyDefinition: Send + Sync + 'static {
    type Key;
}

pub trait Key<TKey, TArg> {
    type Arg;

    fn to_val(&self) -> String;
}

impl KeyDefinition for &'static str {
    type Key = &'static str;
}

impl<TArg> Key<&'static str, TArg> for &'static str {
    type Arg = TArg;

    fn to_val(&self) -> String {
        self.to_string()
    }
}
