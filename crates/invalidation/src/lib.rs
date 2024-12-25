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

use rspc::{Extension, ProcedureStream, Procedures};
use serde::Serialize;

#[derive(Default)]
struct State {
    closures:
        Vec<Arc<dyn Fn(&dyn Any, &mut dyn Any, &dyn Any, &mut Vec<ProcedureStream>) + Send + Sync>>,
}

#[derive(Debug)] // TODO: Traits but only if the generic also has the trait.
pub enum Invalidate<T> {
    None,
    // TODO: Discuss how `Any` is less efficient because it invalidates instead of pushing new data.
    Any,
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

    pub fn with<TCtx, TInput, TResult>(
        // TODO: With multiple middleware how do we enforce we have the first layers `TInput`?
        handler: impl Fn(&E) -> Invalidate<TInput> + Send + Sync + 'static,
    ) -> Extension<TCtx, TInput, TResult>
    where
        TCtx: Send + 'static,
        TInput: Serialize + Send + 'static,
        TResult: Send + 'static,
    {
        let handler = Arc::new(handler);
        Extension::new().setup(|state, meta| {
            // TODO: Error out on mutations or subscriptions due to concerns about safety.

            state
                .get_mut_or_init(|| State::default())
                .closures
                .push(Arc::new(move |event, ctx, procedures, streams| {
                    // TODO: error handling downcast.
                    //  - Can we detect the error on startup and not at runtime?
                    //  - Can we throw onto `Router::build`'s `Result` instead of panicing?
                    let ctx: TCtx = ctx.downcast_mut::<Option<_>>().unwrap().take().unwrap();
                    let event: &E = event.downcast_ref().unwrap();
                    let procedures: &Procedures<TCtx> = procedures.downcast_ref().unwrap();

                    match handler(event) {
                        Invalidate::None => {
                            println!("{:?} {:?}", meta.name(), "NONE"); // TODO
                        }
                        Invalidate::Any => {
                            println!("{:?} {:?}", meta.name(), "ALL"); // TODO
                            todo!(); // TODO: make it work
                        }
                        Invalidate::One(input) => {
                            println!("{:?} {:?}", meta.name(), "ONE"); // TODO

                            // TODO: Avoid `serde_json::Value`?
                            let input: serde_json::Value = serde_json::to_value(&input).unwrap();

                            // let name = meta.name();
                            let name = "sfmPost"; // TODO: Don't do this once `meta.name()` is correct.

                            if let Some(procedure) = procedures.get(name) {
                                streams.push(procedure.exec_with_deserializer(ctx, input));
                            } else {
                                println!("Procedure not found!"); // TODO: Silently fail in future.
                            }
                        }
                        Invalidate::Many(inputs) => {
                            println!("{:?} {:?}", meta.name(), "MANY");
                            todo!();
                        }
                    };
                }));
        })
    }
}

// TODO: The return type does lack info about which procedure is running
// TODO: Should `TCtx` clone vs taking function. This is easier so doing it for now.
pub fn queue<TCtx: Clone + 'static, E: 'static>(
    invalidator: &Invalidator<E>,
    ctx: TCtx,
    procedures: &Procedures<TCtx>,
) -> Vec<ProcedureStream> {
    let mut streams = Vec::new();

    if let Some(state) = procedures.state().get::<State>() {
        let mut invalidated = invalidator
            .invalidated
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        for event in invalidated.drain(..) {
            for closure in &state.closures {
                closure(&event, &mut Some(ctx.clone()), procedures, &mut streams);
            }
        }
    }

    streams
}
