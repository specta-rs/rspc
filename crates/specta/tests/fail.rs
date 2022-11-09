use serde::Serialize;
use specta::Type;

#[cfg(feature = "fail")]
mod unflattenable_enums {
    use super::*;

    // Enums can only flatten if
    // at least one of their variants can flatten

    #[derive(Serialize, Type)]
    enum UnitExternal {
        Unit,
    }

    #[derive(Serialize, Type)]
    enum UnnamedMultiExternal {
        UnnamedMulti(String, String),
    }

    #[derive(Serialize, Type)]
    struct FlattenExternal {
        #[serde(flatten)]
        unit: UnitExternal,
        #[serde(flatten)]
        unnamed_multi: UnnamedMultiExternal,
    }

    #[derive(Serialize, Type)]
    #[serde(untagged)]
    enum UnnamedUntagged {
        Unnamed(String),
    }

    #[derive(Serialize, Type)]
    #[serde(untagged)]
    enum UnnamedMultiUntagged {
        Unnamed(String, String),
    }

    #[derive(Serialize, Type)]
    struct FlattenUntagged {
        #[serde(flatten)]
        unnamed: UnnamedUntagged,
        #[serde(flatten)]
        unnamed_multi: UnnamedMultiUntagged,
    }

    // Adjacent can always flatten

    #[derive(Serialize, Type)]
    #[serde(tag = "tag")]
    enum UnnamedInternal {
        Unnamed(String),
    }

    // Internal can't be used with unnamed multis

    #[derive(Serialize, Type)]
    struct FlattenInternal {
        #[serde(flatten)]
        unnamed: UnnamedInternal,
    }
}
