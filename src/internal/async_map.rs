//! A standard interface for an async map types. This could probs become it's own crate but we can't offer the `get` method using the stdlib API due to lifetime issues, so idk.
//! To understand why we use [`nougat`](nougat) I would recommend reading the [blog post](https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats#hrtb-supertrait) about the system behind it.

use std::{
    collections::HashMap,
    future::{ready, Future, Ready},
};

use nougat::gat;
use tokio::sync::oneshot;

use super::jsonrpc::RequestId;

pub use nougat::gat as _nougat_gat;

// TODO: Generic on `K` and `V`
#[gat]
pub trait AsyncMap: Send {
    type ContainsKeyFut<'a>: Future<Output = bool> + Send + 'a;
    type InsertFut<'a>: Future<Output = Option<oneshot::Sender<()>>> + Send + 'a;
    type RemoveFut<'a>: Future<Output = Option<oneshot::Sender<()>>> + Send + 'a;

    fn contains_key<'a>(&'a self, k: &'a RequestId) -> Self::ContainsKeyFut<'a>;
    fn insert<'a>(&'a mut self, k: RequestId, v: oneshot::Sender<()>) -> Self::InsertFut<'a>;
    fn remove<'a>(&'a mut self, k: &'a RequestId) -> Self::RemoveFut<'a>;
}

#[gat]
impl AsyncMap for HashMap<RequestId, oneshot::Sender<()>> {
    type ContainsKeyFut<'a> = Ready<bool>;
    type InsertFut<'a> = Ready<Option<oneshot::Sender<()>>>;
    type RemoveFut<'a> = Ready<Option<oneshot::Sender<()>>>;

    fn contains_key<'a>(&'a self, k: &'a RequestId) -> Self::ContainsKeyFut<'a> {
        ready(HashMap::contains_key(self, k))
    }

    fn insert<'a>(&'a mut self, k: RequestId, v: oneshot::Sender<()>) -> Self::InsertFut<'a> {
        ready(HashMap::insert(self, k, v))
    }

    fn remove<'a>(&'a mut self, k: &'a RequestId) -> Self::RemoveFut<'a> {
        ready(HashMap::remove(self, k))
    }
}

// TODO: Behind feature flag
mod futures_locks_impl {
    use std::{
        collections::HashMap,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    };

    use futures_locks::MutexFut;
    use nougat::gat;
    use tokio::sync::oneshot;

    use crate::internal::jsonrpc::RequestId;

    use super::{AsyncMap, AsyncMapඞContainsKeyFut, AsyncMapඞInsertFut, AsyncMapඞRemoveFut};

    pub struct FuturesLocksContainsKeyFut<'a> {
        k: &'a RequestId,
        f: MutexFut<HashMap<RequestId, oneshot::Sender<()>>>,
    }

    impl<'k> Future for FuturesLocksContainsKeyFut<'k> {
        type Output = bool;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match Pin::new(&mut this.f).poll(cx) {
                Poll::Ready(map) => Poll::Ready(map.contains_key(this.k)),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    pub struct FuturesLocksInsertFut {
        kv: Option<(RequestId, oneshot::Sender<()>)>,
        f: MutexFut<HashMap<RequestId, oneshot::Sender<()>>>,
    }

    impl Future for FuturesLocksInsertFut {
        type Output = Option<oneshot::Sender<()>>;

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

    pub struct FuturesLocksRemoveFut<'a> {
        k: &'a RequestId,
        f: MutexFut<HashMap<RequestId, oneshot::Sender<()>>>,
    }

    impl<'a> Future for FuturesLocksRemoveFut<'a> {
        type Output = Option<oneshot::Sender<()>>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            match Pin::new(&mut this.f).poll(cx) {
                Poll::Ready(mut map) => Poll::Ready(map.remove(this.k)),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    #[gat]
    impl AsyncMap for futures_locks::Mutex<HashMap<RequestId, oneshot::Sender<()>>> {
        type ContainsKeyFut<'a> = FuturesLocksContainsKeyFut<'a>;
        type InsertFut<'a> = FuturesLocksInsertFut;
        type RemoveFut<'a> = FuturesLocksRemoveFut<'a>;

        fn contains_key<'a>(&'a self, k: &'a RequestId) -> Self::ContainsKeyFut<'a> {
            FuturesLocksContainsKeyFut { k, f: self.lock() }
        }

        fn insert<'a>(&'a mut self, k: RequestId, v: oneshot::Sender<()>) -> Self::InsertFut<'a> {
            FuturesLocksInsertFut {
                kv: Some((k, v)),
                f: self.lock(),
            }
        }

        fn remove<'a>(&'a mut self, k: &'a RequestId) -> Self::RemoveFut<'a> {
            FuturesLocksRemoveFut { k, f: self.lock() }
        }
    }
}

pub use futures_locks_impl::*;
