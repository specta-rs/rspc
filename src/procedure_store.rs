use std::{borrow::Cow, convert::Infallible};

use specta::{ts, DataType, DataTypeFrom, NamedDataType, StructType, Type, TypeMap};

use crate::{layer::DynLayer, ProcedureDef};

// TODO: Rename this
#[deprecated]
pub struct ProcedureTodo<TCtx> {
    // TODO: Back to `pub(crate)`
    pub exec: Box<dyn DynLayer<TCtx>>,
    pub ty: ProcedureDef,
}

// TODO: Using track caller style thing for the panics in this function
// pub fn build<TCtx>(
//     key: Cow<'static, str>,
//     ctx: &mut Router2<TCtx>,
//     kind: ProcedureKind,
//     layer: impl Layer<TCtx> + 'static,
// ) where
//     TCtx: Send + 'static,
// {
//     let (map, type_name) = match kind {
//         ProcedureKind::Query => (&mut ctx.queries, "query"),
//         ProcedureKind::Mutation => (&mut ctx.mutations, "mutation"),
//         ProcedureKind::Subscription => (&mut ctx.subscriptions, "subscription"),
//     };

//     let key_org = key;
//     let key = key_org.to_string();
//     let type_def = layer
//         .into_procedure_def(key_org, &mut ctx.typ_store)
//         .expect("error exporting types");

//     // TODO: Cleanup this logic and do better router merging
//     #[allow(clippy::panic)]
//     if key.is_empty() || key == "ws" || key.starts_with("rpc.") || key.starts_with("rspc.") {
//         panic!("rspc error: attempted to create {type_name} operation named '{key}', however this name is not allowed.");
//     }

//     #[allow(clippy::panic)]
//     if map.contains_key(&key) {
//         panic!("rspc error: {type_name} operation already has resolver with name '{key}'");
//     }

//     map.insert(
//         key,
//         ProcedureTodo {
//             exec: boxed(layer),
//             ty: type_def,
//         },
//     );
// }
