use std::{
    boxed::Box,
    collections::HashMap,
    // collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::Hash,
};

// use serde::Serialize;
use serde_json::{json, Value};
// use specta::Type;

/// TODO
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ObjectRef {
    pub id: crate::Value,
    pub ty: &'static str,
}

impl From<ObjectRef> for Value {
    fn from(value: ObjectRef) -> Self {
        json!({
            "$id": value.id,
            "$ty": value.ty,
        })
    }
}

/// TODO
pub type RefMap = HashMap<ObjectRef, Value>;

/// TODO
pub trait Object: 'static {
    /// is used to determine the type of the current object. It will define to Rust's debug type name but you SHOULD override it.
    fn type_name() -> &'static str;

    /// is used to determine the unique identifier for this object. The id must be unique between all objects of the same type.
    fn id(&self) -> Result<serde_json::Value, serde_json::Error>;

    /// TODO
    fn normalize(self, refs: &mut RefMap) -> Result<serde_json::Value, serde_json::Error>;
}

impl<T: Object> Object for Vec<T> {
    fn type_name() -> &'static str {
        T::type_name()
    }

    #[allow(clippy::panic_in_result_fn)]
    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self, refs: &mut RefMap) -> Result<serde_json::Value, serde_json::Error> {
        let mut vec = Vec::with_capacity(self.len());
        for item in self {
            vec.push(item.normalize(refs)?);
        }
        Ok(Value::Array(vec))
    }
}

impl<T: Object> Object for Box<T> {
    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        <T as Object>::id(self)
    }

    fn normalize(self, refs: &mut RefMap) -> Result<serde_json::Value, serde_json::Error> {
        <T as Object>::normalize(*self, refs)
    }
}

// impl<T: Object> Object for Option<T> {
//     type NormalizedResult = Option<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         match self {
//             Some(v) => Ok(Some(<T as Object>::normalize(v)?)),
//             None => Ok(None),
//         }
//     }
// }

// #[derive(Serialize, Type)]
// pub struct NormalizedVecDeque<T> {
//     #[serde(rename = "$type")]
//     __type: &'static str,
//     edges: VecDeque<T>,
// }

// impl<T: Object> Object for VecDeque<T> {
//     type NormalizedResult = NormalizedVecDeque<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         Ok(NormalizedVecDeque {
//             __type: Self::type_name(),
//             edges: self
//                 .into_iter()
//                 .map(|v| v.normalize())
//                 .collect::<Result<VecDeque<_>, _>>()?,
//         })
//     }
// }

// #[derive(Serialize, Type)]
// pub struct NormalizedBinaryHeap<T>
// where
//     T: Ord,
// {
//     #[serde(rename = "$type")]
//     __type: &'static str,
//     edges: BinaryHeap<T>,
// }

// impl<T: Object> Object for BinaryHeap<T>
// where
//     T::NormalizedResult: Ord,
// {
//     type NormalizedResult = NormalizedBinaryHeap<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         Ok(NormalizedBinaryHeap {
//             __type: Self::type_name(),
//             edges: self
//                 .into_iter()
//                 .map(|v| v.normalize())
//                 .collect::<Result<BinaryHeap<_>, _>>()?,
//         })
//     }
// }

// #[derive(Serialize, Type)]
// pub struct NormalizedLinkedList<T>
// where
//     T: Ord,
// {
//     #[serde(rename = "$type")]
//     __type: &'static str,
//     edges: LinkedList<T>,
// }

// impl<T: Object> Object for LinkedList<T>
// where
//     T::NormalizedResult: Ord,
// {
//     type NormalizedResult = NormalizedBinaryHeap<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         Ok(NormalizedBinaryHeap {
//             __type: Self::type_name(),
//             edges: self
//                 .into_iter()
//                 .map(|v| v.normalize())
//                 .collect::<Result<BinaryHeap<_>, _>>()?,
//         })
//     }
// }

// #[derive(Serialize, Type)]
// pub struct NormalizedHashSet<T>
// where
//     T: Hash + Eq,
// {
//     #[serde(rename = "$type")]
//     __type: &'static str,
//     edges: HashSet<T>,
// }

// impl<T: Object> Object for HashSet<T>
// where
//     T::NormalizedResult: Hash + Eq,
// {
//     type NormalizedResult = NormalizedHashSet<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         Ok(NormalizedHashSet {
//             __type: Self::type_name(),
//             edges: self
//                 .into_iter()
//                 .map(|v| v.normalize())
//                 .collect::<Result<HashSet<_>, _>>()?,
//         })
//     }
// }

// #[derive(Serialize, Type)]
// pub struct NormalizedBTreeSet<T>
// where
//     T: Ord,
// {
//     #[serde(rename = "$type")]
//     __type: &'static str,
//     edges: BTreeSet<T>,
// }

// impl<T: Object> Object for BTreeSet<T>
// where
//     T::NormalizedResult: Ord,
// {
//     type NormalizedResult = NormalizedBTreeSet<T::NormalizedResult>;

//     fn type_name() -> &'static str {
//         <T as Object>::type_name()
//     }

//     fn id(&self) -> Result<Value, serde_json::Error> {
//         unreachable!();
//     }

//     fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
//         Ok(NormalizedBTreeSet {
//             __type: Self::type_name(),
//             edges: self
//                 .into_iter()
//                 .map(|v| v.normalize())
//                 .collect::<Result<BTreeSet<_>, _>>()?,
//         })
//     }
// }

// // &'a [T]
// // [T; N]

// // TODO: IndexSet
// // TODO: Implmenting for tuples???
// // TODO: Normalising Map types -> BTreeMap, HashMap, serde_json::Map, IndexMap
