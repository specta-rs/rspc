//! A standard interface for an async map types. This could probs become it's own crate but we can't offer the `get` method using the stdlib API due to lifetime issues, so idk.
//! To understand why we use [`nougat`](nougat) I would recommend reading the [blog post](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats#hrtb-supertrait) about the system behind it.

use std::{
    collections::HashMap,
    future::{ready, Future, Ready},
    hash::Hash,
};

use nougat::gat;

pub use nougat::gat as _nougat_gat;

#[gat]
pub trait AsyncMap<K, V>: Send {
    type ContainsKeyFut<'a>: Future<Output = bool> + Send + 'a
    where
        K: 'a,
        V: 'a;
    type InsertFut<'a>: Future<Output = Option<V>> + Send + 'a
    where
        K: 'a,
        V: 'a;
    type RemoveFut<'a>: Future<Output = Option<V>> + Send + 'a
    where
        K: 'a,
        V: 'a;

    fn contains_key<'a>(&'a self, k: &'a K) -> Self::ContainsKeyFut<'a>;
    fn insert<'a>(&'a mut self, k: K, v: V) -> Self::InsertFut<'a>;
    fn remove<'a>(&'a mut self, k: &'a K) -> Self::RemoveFut<'a>;
}

#[gat]
impl<K, V> AsyncMap<K, V> for HashMap<K, V>
where
    K: Eq + Hash + Send,
    V: Send,
{
    type ContainsKeyFut<'a> = Ready<bool> where K: 'a, V: 'a;
    type InsertFut<'a> = Ready<Option<V>> where K: 'a, V: 'a;
    type RemoveFut<'a> = Ready<Option<V>> where K: 'a, V: 'a;

    fn contains_key<'a>(&'a self, k: &'a K) -> Self::ContainsKeyFut<'a> {
        ready(HashMap::contains_key(self, k))
    }

    fn insert<'a>(&'a mut self, k: K, v: V) -> Self::InsertFut<'a> {
        ready(HashMap::insert(self, k, v))
    }

    fn remove<'a>(&'a mut self, k: &'a K) -> Self::RemoveFut<'a> {
        ready(HashMap::remove(self, k))
    }
}

// If the lifetimes all explode this is without a doubt involved. It kinda has to exist for the Tauri plugin to work without a bit of duplicate code within the core.
#[gat]
impl<'this, K, V> AsyncMap<K, V> for &'this mut HashMap<K, V>
where
    K: Eq + Hash + Send,
    V: Send,
{
    type ContainsKeyFut<'a> = Ready<bool> where K: 'a, V: 'a;
    type InsertFut<'a> = Ready<Option<V>> where K: 'a, V: 'a;
    type RemoveFut<'a> = Ready<Option<V>> where K: 'a, V: 'a;

    fn contains_key<'a>(&'a self, k: &'a K) -> Self::ContainsKeyFut<'a> {
        ready(HashMap::contains_key(self, k))
    }

    fn insert<'a>(&'a mut self, k: K, v: V) -> Self::InsertFut<'a> {
        ready(HashMap::insert(self, k, v))
    }

    fn remove<'a>(&'a mut self, k: &'a K) -> Self::RemoveFut<'a> {
        ready(HashMap::remove(self, k))
    }
}

// TODO: Behind feature flag
mod futures_locks_impl {
    use std::{
        collections::HashMap,
        future::Future,
        hash::Hash,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    };

    use futures_locks::MutexFut;
    use nougat::gat;

    use super::{AsyncMap, AsyncMapඞContainsKeyFut, AsyncMapඞInsertFut, AsyncMapඞRemoveFut};

    pub struct FuturesLocksContainsKeyFut<'a, K, V> {
        k: &'a K,
        f: MutexFut<HashMap<K, V>>,
    }

    impl<'k, K, V> Future for FuturesLocksContainsKeyFut<'k, K, V>
    where
        K: Eq + Hash + Unpin,
    {
        type Output = bool;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match Pin::new(&mut this.f).poll(cx) {
                Poll::Ready(map) => Poll::Ready(map.contains_key(this.k)),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    pub struct FuturesLocksInsertFut<K, V> {
        kv: Option<(K, V)>,
        f: MutexFut<HashMap<K, V>>,
    }

    impl<K, V> Future for FuturesLocksInsertFut<K, V>
    where
        K: Eq + Hash + Unpin,
        V: Unpin,
    {
        type Output = Option<V>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match Pin::new(&mut this.f).poll(cx) {
                Poll::Ready(mut map) => {
                    let (k, v) = this.kv.take().expect("future was polled after completion");
                    Poll::Ready(map.insert(k, v))
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }

    pub struct FuturesLocksRemoveFut<'a, K, V> {
        k: &'a K,
        f: MutexFut<HashMap<K, V>>,
    }

    impl<'a, K, V> Future for FuturesLocksRemoveFut<'a, K, V>
    where
        K: Eq + Hash,
    {
        type Output = Option<V>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match Pin::new(&mut this.f).poll(cx) {
                Poll::Ready(mut map) => Poll::Ready(map.remove(this.k)),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    #[gat]
    impl<K, V> AsyncMap<K, V> for futures_locks::Mutex<HashMap<K, V>>
    where
        K: Eq + Hash + Send + Sync + Unpin,
        V: Send + Unpin,
    {
        type ContainsKeyFut<'a> = FuturesLocksContainsKeyFut<'a, K, V> where K: 'a, V: 'a;
        type InsertFut<'a> = FuturesLocksInsertFut<K, V> where K: 'a, V: 'a;
        type RemoveFut<'a> = FuturesLocksRemoveFut<'a, K, V> where K: 'a, V: 'a;

        fn contains_key<'a>(&'a self, k: &'a K) -> Self::ContainsKeyFut<'a> {
            FuturesLocksContainsKeyFut { k, f: self.lock() }
        }

        fn insert<'a>(&'a mut self, k: K, v: V) -> Self::InsertFut<'a> {
            FuturesLocksInsertFut {
                kv: Some((k, v)),
                f: self.lock(),
            }
        }

        fn remove<'a>(&'a mut self, k: &'a K) -> Self::RemoveFut<'a> {
            FuturesLocksRemoveFut { k, f: self.lock() }
        }
    }

    #[gat]
    impl<K, V> AsyncMap<K, V> for Arc<futures_locks::Mutex<HashMap<K, V>>>
    where
        K: Eq + Hash + Send + Sync + Unpin,
        V: Send + Unpin,
    {
        type ContainsKeyFut<'a> = FuturesLocksContainsKeyFut<'a, K, V> where K: 'a, V: 'a;
        type InsertFut<'a> = FuturesLocksInsertFut<K, V> where K: 'a, V: 'a;
        type RemoveFut<'a> = FuturesLocksRemoveFut<'a, K, V> where K: 'a, V: 'a;

        fn contains_key<'a>(&'a self, k: &'a K) -> Self::ContainsKeyFut<'a> {
            FuturesLocksContainsKeyFut { k, f: self.lock() }
        }

        fn insert<'a>(&'a mut self, k: K, v: V) -> Self::InsertFut<'a> {
            FuturesLocksInsertFut {
                kv: Some((k, v)),
                f: self.lock(),
            }
        }

        fn remove<'a>(&'a mut self, k: &'a K) -> Self::RemoveFut<'a> {
            FuturesLocksRemoveFut { k, f: self.lock() }
        }
    }
}

pub use futures_locks_impl::*;
