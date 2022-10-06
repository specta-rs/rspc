use std::{
    boxed::Box,
    collections::{BTreeSet, BinaryHeap, HashSet, LinkedList, VecDeque},
    hash::Hash,
};

use serde::Serialize;
use serde_json::Value;
use specta::Type;

/// TODO
pub trait Object: 'static {
    type NormalizedResult: Serialize + Type + Send;

    /// is used to determine the type of the current object. It will define to Rust's debug type name but you SHOULD override it.
    fn type_name() -> &'static str;

    /// is used to determine the unique identifier for this object. The id must be unique between all objects of the same type.
    fn id(&self) -> Result<Value, serde_json::Error>;

    /// TODO
    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error>;
}

#[derive(Serialize, Type)]
pub struct NormalizedVec<T> {
    __type: &'static str,
    edges: Vec<T>,
}

impl<T: Object> Object for Vec<T> {
    type NormalizedResult = NormalizedVec<T::NormalizedResult>;

    fn type_name() -> &'static str {
        T::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedVec {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl<T: Object> Object for Box<T> {
    type NormalizedResult = T::NormalizedResult;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        <T as Object>::id(&self)
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        <T as Object>::normalize(*self)
    }
}

impl<T: Object> Object for Option<T> {
    type NormalizedResult = Option<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        match self {
            Some(v) => Ok(Some(<T as Object>::normalize(v)?)),
            None => Ok(None),
        }
    }
}

#[derive(Serialize, Type)]
pub struct NormalizedVecDeque<T> {
    __type: &'static str,
    edges: VecDeque<T>,
}

impl<T: Object> Object for VecDeque<T> {
    type NormalizedResult = NormalizedVecDeque<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedVecDeque {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<VecDeque<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Type)]
pub struct NormalizedBinaryHeap<T>
where
    T: Ord,
{
    __type: &'static str,
    edges: BinaryHeap<T>,
}

impl<T: Object> Object for BinaryHeap<T>
where
    T::NormalizedResult: Ord,
{
    type NormalizedResult = NormalizedBinaryHeap<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedBinaryHeap {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<BinaryHeap<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Type)]
pub struct NormalizedLinkedList<T>
where
    T: Ord,
{
    __type: &'static str,
    edges: LinkedList<T>,
}

impl<T: Object> Object for LinkedList<T>
where
    T::NormalizedResult: Ord,
{
    type NormalizedResult = NormalizedBinaryHeap<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedBinaryHeap {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<BinaryHeap<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Type)]
pub struct NormalizedHashSet<T>
where
    T: Hash + Eq,
{
    __type: &'static str,
    edges: HashSet<T>,
}

impl<T: Object> Object for HashSet<T>
where
    T::NormalizedResult: Hash + Eq,
{
    type NormalizedResult = NormalizedHashSet<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedHashSet {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<HashSet<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Type)]
pub struct NormalizedBTreeSet<T>
where
    T: Ord,
{
    __type: &'static str,
    edges: BTreeSet<T>,
}

impl<T: Object> Object for BTreeSet<T>
where
    T::NormalizedResult: Ord,
{
    type NormalizedResult = NormalizedBTreeSet<T::NormalizedResult>;

    fn type_name() -> &'static str {
        <T as Object>::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedBTreeSet {
            __type: Self::type_name(),
            edges: self
                .into_iter()
                .map(|v| v.normalize())
                .collect::<Result<BTreeSet<_>, _>>()?,
        })
    }
}

// &'a [T]
// [T; N]

// TODO: IndexSet
// TODO: Implmenting for tuples???
// TODO: Normalising Map types -> BTreeMap, HashMap, serde_json::Map, IndexMap
