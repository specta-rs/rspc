#![allow(unsafe_code)]
///! TODO: Explain this abstraction
// TODO: The goal is this can only know about the inner `T` and not `Arc<T>` but also ensure `ArcRef<T>` is not dropped until it's safe.
// TODO: This abstraction is safe because `&SomeDerivedType` is tied to the ownership of `Arc<T>` of which it was derived from.
// TODO: This is basically required for queueing an rspc subscription onto it's own task which with Tokio requires `'static`.
// TODO: This whole thing is really similar to the `owning_ref` crate but I want to erase the `T` from `Arc<T>` which is done through the `drop` function pointer.
use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

use serde_json::Value;

use crate::{
    body::Body,
    middleware::{ProcedureKind, RequestContext},
    router_builder::ProcedureMap,
    Router,
};

use super::RequestData;

pub(crate) struct ArcRef<T: 'static> {
    // The lifetime here is invalid. This type is actually valid as long as the `Arc` in `self.mem` is ok.
    val: T,
    arc: *const (),
    drop: fn(*const ()),
}

unsafe impl<T: Send + 'static> Send for ArcRef<T> {}
unsafe impl<T: Sync + 'static> Sync for ArcRef<T> {}

impl<T: 'static> ArcRef<T> {
    /// The caller in-charge of ensuring the `val` is derived from `val`.
    unsafe fn new<A: 'static>(arc: Arc<A>, val: T) -> Self {
        debug_assert_eq!(
            size_of::<*const ()>(),
            size_of::<*const T>(),
            "pointer size mismatch"
        );

        // let val = (func)(&arc);

        // // SAFETY: We are forcing this value into a `'static` lifetime because it's lifetime is derived from the `Arc` which itself has a `'static` lifetime.
        // // SAFETY: For this to remain safe we hold the `arc` on self so the memory can't be deallocated while we are having the `'static` reference.
        // // SAFETY: We also ensure the `'static` never escapes the `ArcRef` by requiring `Deref` to use the inner value which itself ties the usage of the lifetime of `ArcRef`.
        // let val = unsafe { std::mem::transmute::<&mut T, &'static mut T>(&mut val) };

        let arc2 = arc.clone(); // TODO: Avoid clone?

        Self {
            val,
            arc: Arc::into_raw(arc2) as *const (),
            drop: |ptr| {
                // SAFETY: Reconstruct the arc from the pointer so Rust can decrement the strong count and potentially drop the memory if required.
                drop(unsafe { Arc::from_raw(ptr as *const A) });
            },
        }
    }
}

impl<T> Deref for ArcRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T> DerefMut for ArcRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for ArcRef<T> {
    fn drop(&mut self) {
        (self.drop)(self.arc);
    }
}

// BELOW ARE HELPERS FOR SAFELY GETTING THE `ArcRef`

fn get_procedure<TCtx: 'static>(
    router: Arc<Router<TCtx>>,
    ctx: TCtx,
    data: RequestData,
    kind: ProcedureKind,
    procedures: fn(&Arc<Router<TCtx>>) -> &ProcedureMap<TCtx>,
) -> Option<ArcRef<Pin<Box<dyn Body + Send>>>> {
    let req = RequestContext::new(data.id, kind, data.path);
    let procedures: *const _ = procedures(&router);
    // SAFETY: This is basically extending the lifetime to `'static`. This is safe cause we hold on to the `Arc` which owns the memory at the same time.
    let procedures = unsafe { &*procedures };

    let stream = procedures
        .get(req.path.as_ref())?
        .exec
        .dyn_call(ctx, data.input.unwrap_or(Value::Null), req)
        .into();

    Some(unsafe { ArcRef::new(router, stream) })
}

struct SourceProxy<T: 'static>(&'static T);

impl<T> Deref for SourceProxy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// unsafe fn get_arc_ref<TSource, TItem>(
//     source: Arc<TSource>,
//     func: impl FnOnce(&TSource) -> TItem,
// ) -> ArcRef<TItem> {
//     let source_arc = source;

//     let source: *const _ = &*source;
//     let source: &'static _ = unsafe { &*source };

//     let proxy = SourceProxy(source);

//     let item = func(&proxy);

//     unsafe { ArcRef::new(source_arc, item) }
// }

// fn get_procedure<TCtx: 'static>(
//     router: Arc<Router<TCtx>>,
//     ctx: TCtx,
//     data: RequestData,
//     kind: ProcedureKind,
//     procedures: fn(&Router<TCtx>) -> &ProcedureMap<TCtx>,
// ) -> ArcRef<Option<Box<dyn Body + Send>>> {
//     unsafe {
//         get_arc_ref(router, |router| {
//             let _: &'static _ = router;

//             let procedures = procedures(router);

//             let req = RequestContext::new(data.id, kind, data.path);
//             let stream = procedures.get(req.path.as_ref())?.exec.dyn_call(
//                 ctx,
//                 data.input.unwrap_or(Value::Null),
//                 req,
//             );

//             Some(stream)
//         })
//     }
// }

pub(crate) fn get_query<TCtx: 'static>(
    router: Arc<Router<TCtx>>,
    ctx: TCtx,
    data: RequestData,
) -> Option<ArcRef<Pin<Box<dyn Body + Send>>>> {
    get_procedure(router, ctx, data, ProcedureKind::Query, |router| {
        &router.queries
    })
}

pub(crate) fn get_mutation<TCtx: 'static>(
    router: Arc<Router<TCtx>>,
    ctx: TCtx,
    data: RequestData,
) -> Option<ArcRef<Pin<Box<dyn Body + Send>>>> {
    get_procedure(router, ctx, data, ProcedureKind::Mutation, |router| {
        &router.mutations
    })
}

pub(crate) fn get_subscription<TCtx: 'static>(
    router: Arc<Router<TCtx>>,
    ctx: TCtx,
    data: RequestData,
) -> Option<ArcRef<Pin<Box<dyn Body + Send>>>> {
    get_procedure(router, ctx, data, ProcedureKind::Subscription, |router| {
        &router.subscriptions
    })
}
