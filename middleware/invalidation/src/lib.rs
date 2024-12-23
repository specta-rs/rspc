//! rspc-invalidation: Real-time invalidation support for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

use std::{
    any::Any,
    sync::{Arc, Mutex, PoisonError},
};

use rspc::{middleware::Middleware, Extension, ProcedureStream, Procedures};

#[derive(Default)]
struct State {
    closures: Vec<Arc<dyn Fn(&mut dyn Any) -> () + Send + Sync>>,
}

#[derive(Debug)] // TODO: Traits but only if the generic also has the trait.
pub enum Invalidate<T> {
    None,
    All,
    One(T),
    Many(Vec<T>),
}

pub struct Invalidator<E> {
    // TODO: I don't like this but solving that is *really* hard.
    invalidated: Arc<Mutex<Vec<E>>>,
}

// TODO: `Debug` impl

impl<E> Default for Invalidator<E> {
    fn default() -> Self {
        Self {
            invalidated: Default::default(),
        }
    }
}

impl<E> Clone for Invalidator<E> {
    fn clone(&self) -> Self {
        Self {
            invalidated: self.invalidated.clone(),
        }
    }
}

impl<E: 'static> Invalidator<E> {
    // TODO: Taking `&mut self` will cause major problems with people doing `Arc<TCtx>`.
    pub fn invalidate(&self, event: E) {
        self.invalidated
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    }

    // pub fn mw<TError, TCtx, TInput, TResult>(
    //     // TODO: With multiple middleware how do we enforce we have the first layers `TInput`?
    //     handler: impl Fn(&E) -> Invalidate<TInput> + Send + Sync + 'static,
    // ) -> Middleware<TError, TCtx, TInput, TResult>
    // where
    //     TError: Send + 'static,
    //     TCtx: Send + 'static,
    //     TInput: Send + 'static,
    //     TResult: Send + 'static,
    // {
    //     let handler = Arc::new(handler);
    //     Middleware::new(move |ctx: TCtx, input: TInput, next| async move {
    //         let result = next.exec(ctx, input).await;
    //         result
    //     })
    //     .setup(|state, meta| {
    //         // TODO: Error out on mutations or subscriptions due to concerns about safety.

    //         state
    //             .get_mut_or_init(|| State::default())
    //             .closures
    //             .push(Arc::new(move |event| {
    //                 match handler(event.downcast_ref().unwrap()) {
    //                     Invalidate::None => println!("{:?} {:?}", meta.name(), "NONE"),
    //                     // TODO: Make these work properly
    //                     Invalidate::All => println!("{:?} {:?}", meta.name(), "ALL"),
    //                     Invalidate::One(input) => println!("{:?} {:?}", meta.name(), "ONE"),
    //                     Invalidate::Many(inputs) => println!("{:?} {:?}", meta.name(), "MANY"),
    //                 }
    //             }));
    //     })
    // }

    pub fn with<TCtx, TInput, TResult>(
        // TODO: With multiple middleware how do we enforce we have the first layers `TInput`?
        handler: impl Fn(&E) -> Invalidate<TInput> + Send + Sync + 'static,
    ) -> Extension<TCtx, TInput, TResult>
    where
        TCtx: Send + 'static,
        TInput: Send + 'static,
        TResult: Send + 'static,
    {
        let handler = Arc::new(handler);
        Extension::new().setup(|state, meta| {
            // TODO: Error out on mutations or subscriptions due to concerns about safety.

            state
                .get_mut_or_init(|| State::default())
                .closures
                .push(Arc::new(move |event| {
                    match handler(event.downcast_ref().unwrap()) {
                        Invalidate::None => println!("{:?} {:?}", meta.name(), "NONE"),
                        // TODO: Make these work properly
                        Invalidate::All => println!("{:?} {:?}", meta.name(), "ALL"),
                        Invalidate::One(input) => println!("{:?} {:?}", meta.name(), "ONE"),
                        Invalidate::Many(inputs) => println!("{:?} {:?}", meta.name(), "MANY"),
                    }
                }));
        })
    }
}

// TODO: The return type does lack info about which procedure is running
pub fn queue<TCtx, E: 'static>(
    invalidator: &Invalidator<E>,
    ctx_fn: impl Fn() -> TCtx,
    procedures: &Procedures<TCtx>,
) -> Vec<ProcedureStream> {
    let mut streams = Vec::new();

    if let Some(state) = procedures.state().get::<State>() {
        let mut invalidated = invalidator
            .invalidated
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        for mut event in invalidated.drain(..) {
            for closure in &state.closures {
                closure(&mut event); // TODO: Take in `streams`, `procedures` and `ctx_fn`
            }
        }
    }

    let keys_to_invalidate = vec!["version"]; // TODO: How to work out which procedures to rerun? -> We need some request scoped data.

    for name in keys_to_invalidate {
        streams.push(
            procedures
                .get(name)
                .unwrap()
                // TODO: Don't deserialize to `serde_json::Value` and make the input type work properly.
                .exec_with_deserializer(ctx_fn(), serde_json::Value::Null),
        );
    }

    streams
}
