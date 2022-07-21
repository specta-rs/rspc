use std::fmt::Debug;

use crate::ExecError;

/// TODO
pub trait KeyDefinition: Sized + Send + 'static {
    type Key: Send + Sync + 'static;
    type KeyRaw: ToString + Clone + Ord + Debug + Send + Sync + 'static; // TODO: Rename this type?

    fn add_prefix(key_raw: Self::KeyRaw, prefix: &'static str) -> Self::KeyRaw;

    fn from_str(key: String) -> Result<Self::KeyRaw, ExecError>;
}

/// TODO
pub trait Key<TKey: KeyDefinition, TArg> {
    type Arg;

    fn to_val(self) -> TKey::KeyRaw;
}

impl KeyDefinition for &'static str {
    type Key = &'static str;
    type KeyRaw = String;

    fn add_prefix(key_raw: String, prefix: &'static str) -> String {
        format!("{}{}", prefix, key_raw)
    }

    fn from_str(key: String) -> Result<Self::KeyRaw, ExecError> {
        Ok(key)
    }
}

impl<TArg> Key<&'static str, TArg> for &'static str {
    type Arg = TArg;

    fn to_val(self) -> String {
        self.to_string()
    }
}

impl KeyDefinition for u32 {
    type Key = u32;
    type KeyRaw = u32;

    fn add_prefix(_key_raw: Self::KeyRaw, _prefix: &'static str) -> Self::KeyRaw {
        todo!("Merging routes is currently only supported for `&'static str` keys! This will be supported in the future!");
    }

    fn from_str(key: String) -> Result<Self::KeyRaw, ExecError> {
        key.parse().map_err(|_| ExecError::OperationNotFound(key))
    }
}

impl<TArg> Key<u32, TArg> for u32 {
    type Arg = TArg;

    fn to_val(self) -> u32 {
        self
    }
}
