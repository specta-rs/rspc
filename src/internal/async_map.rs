//! A standard interface for an async map types. This could probs become it's own crate but we can't offer the `get` method using the stdlib API due to lifetime issues, so idk.

use std::future::Future;

// mod futures_lock_mutex {
//     use std::{
//         collections::HashMap,
//         future::Future,
//         hash::Hash,
//         pin::Pin,
//         task::{Context, Poll},
//     };

//     use futures_locks::MutexFut;

//     use super::AsyncMap;

//     pub struct FuturesLocksContainsKeyFut<'k, K: 'k, V> {
//         k: &'k K,
//         f: MutexFut<HashMap<K, V>>,
//     }

//     impl<'k, K: 'k, V> Future for FuturesLocksContainsKeyFut<'k, K, V>
//     where
//         K: Hash + Eq + Unpin,
//         V: Unpin,
//     {
//         type Output = bool;

//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             let this = self.get_mut();
//             match Pin::new(&mut this.f).poll(cx) {
//                 Poll::Ready(map) => Poll::Ready(map.contains_key(this.k)),
//                 Poll::Pending => Poll::Pending,
//             }
//         }
//     }

//     // // TODO: Revert this
//     // pub struct FuturesLocksContainsKeyFut<K, V> {
//     //     k: K,
//     //     f: MutexFut<HashMap<K, V>>,
//     // }

//     // impl<K, V> Future for FuturesLocksContainsKeyFut<K, V>
//     // where
//     //     K: Hash + Eq + Unpin,
//     //     V: Unpin,
//     // {
//     //     type Output = bool;

//     //     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//     //         let this = self.get_mut();
//     //         match Pin::new(&mut this.f).poll(cx) {
//     //             Poll::Ready(map) => Poll::Ready(map.contains_key(&this.k)),
//     //             Poll::Pending => Poll::Pending,
//     //         }
//     //     }
//     // }

//     pub struct FuturesLocksInsertFut<K, V> {
//         kv: Option<(K, V)>,
//         f: MutexFut<HashMap<K, V>>,
//     }

//     impl<K, V> Future for FuturesLocksInsertFut<K, V>
//     where
//         K: Hash + Eq + Unpin,
//         V: Unpin,
//     {
//         type Output = Option<V>;

//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             let this = self.get_mut();
//             match Pin::new(&mut this.f).poll(cx) {
//                 Poll::Ready(mut map) => {
//                     let (k, v) = this.kv.take().expect("future was polled after completion");
//                     Poll::Ready(map.insert(k, v))
//                 }
//                 Poll::Pending => Poll::Pending,
//             }
//         }
//     }

//     pub struct FuturesLocksRemoveFut<'k, K: 'k, V> {
//         k: &'k K,
//         f: MutexFut<HashMap<K, V>>,
//     }

//     impl<'k, K: 'k, V> Future for FuturesLocksRemoveFut<'k, K, V>
//     where
//         K: Hash + Eq + Unpin,
//         V: Unpin,
//     {
//         type Output = Option<V>;

//         fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//             let this = self.get_mut();
//             match Pin::new(&mut this.f).poll(cx) {
//                 Poll::Ready(mut map) => Poll::Ready(map.remove(this.k)),
//                 Poll::Pending => Poll::Pending,
//             }
//         }
//     }

//     impl<K, V> AsyncMap<K, V> for futures_locks::Mutex<HashMap<K, V>>
//     where
//         K: Hash + Eq + Send + Sync + Unpin,
//         V: Send + Sync + Unpin,
//     {
//         type ContainsKeyFut<'k> = FuturesLocksContainsKeyFut<'k, K, V>
//         where
//             Self: 'k,
//             K: 'k,
//             V: 'k;

//         type InsertFut = FuturesLocksInsertFut<K, V>;

//         type RemoveFut<'k> = FuturesLocksRemoveFut<'k, K, V>
//         where
//             Self: 'k,
//             K: 'k,
//             V: 'k;

//         fn contains_key<'k>(&self, k: &'k K) -> Self::ContainsKeyFut<'k>
//         where
//             V: 'k,
//         {
//             FuturesLocksContainsKeyFut { k, f: self.lock() }
//         }

//         fn insert(&mut self, k: K, v: V) -> Self::InsertFut {
//             FuturesLocksInsertFut {
//                 kv: Some((k, v)),
//                 f: self.lock(),
//             }
//         }

//         fn remove<'k>(&mut self, k: &'k K) -> Self::RemoveFut<'k>
//         where
//             V: 'k,
//         {
//             FuturesLocksRemoveFut { k, f: self.lock() }
//         }
//     }
// }

// pub use futures_lock_mutex::*;

mod std_hash_map {
    use std::{
        collections::HashMap,
        future::{ready, Ready},
        hash::Hash,
    };

    use super::AsyncMap;

    impl<K, V> AsyncMap<K, V> for HashMap<K, V>
    where
        K: Hash + Eq,
        V: Send + Sync,
    {
        type ContainsKeyFut<'k> = Ready<bool>
        where
            Self: 'k,
            K: 'k,
            V: 'k;

        type InsertFut = Ready<Option<V>>;

        type RemoveFut<'k> = Ready<Option<V>>
        where
            Self: 'k,
            K: 'k,
            V: 'k;

        fn contains_key<'k>(&self, k: &'k K) -> Self::ContainsKeyFut<'k>
        where
            V: 'k,
        {
            ready(self.contains_key(k))
        }

        fn contains_key2(&self, k: K) -> Ready<bool> {
            ready(self.contains_key(&k))
        }

        fn insert(&mut self, k: K, v: V) -> Self::InsertFut {
            ready(self.insert(k, v))
        }

        fn remove<'k>(&mut self, k: &'k K) -> Self::RemoveFut<'k>
        where
            V: 'k,
        {
            ready(self.remove(k))
        }
    }
}

pub use std_hash_map::*;

/// TODO
pub trait AsyncMap<K, V> {
    type ContainsKeyFut<'k>: Future<Output = bool> + Send + Sync
    where
        Self: 'k,
        K: 'k,
        V: 'k;

    type InsertFut: Future<Output = Option<V>> + Send + Sync;

    type RemoveFut<'k>: Future<Output = Option<V>> + Send + Sync
    where
        Self: 'k,
        K: 'k,
        V: 'k;

    fn contains_key<'k>(&self, k: &'k K) -> Self::ContainsKeyFut<'k>
    where
        Self: 'k,
        V: 'k;

    fn contains_key2(&self, k: K) -> std::future::Ready<bool>; // TODO: Remove

    fn insert(&mut self, k: K, v: V) -> Self::InsertFut;

    fn remove<'k>(&mut self, k: &'k K) -> Self::RemoveFut<'k>
    where
        V: 'k;
}

// TODO: This works but I suspect it's not helping with the lifetime issues so is disabled
impl<'a, K, V, T: AsyncMap<K, V>> AsyncMap<K, V> for &'a mut T {
    type ContainsKeyFut<'k> = T::ContainsKeyFut<'k>
    where
        Self: 'k,
        K: 'k,
        V: 'k;

    type InsertFut = T::InsertFut;

    type RemoveFut<'k> = T::RemoveFut<'k>
    where
        Self: 'k,
        K: 'k,
        V: 'k;

    fn contains_key<'k>(&self, k: &'k K) -> Self::ContainsKeyFut<'k>
    where
        V: 'k,
        'a: 'k,
    {
        (**self).contains_key(k)
    }

    fn contains_key2(&self, k: K) -> std::future::Ready<bool> {
        (**self).contains_key2(k)
    }

    fn insert(&mut self, k: K, v: V) -> Self::InsertFut {
        (**self).insert(k, v)
    }

    fn remove<'k>(&mut self, k: &'k K) -> Self::RemoveFut<'k>
    where
        V: 'k,
        'a: 'k,
    {
        (**self).remove(k)
    }
}
