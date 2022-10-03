use std::marker::PhantomData;

use futures::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use specta::{DefOpts, Type, TypeDefs};

use crate::{
    internal::{ProcedureDataType, RequestFuture, StreamFuture},
    ExecError, RequestResult,
};

pub trait RequestResolver<TCtx, TMarker>: Send + Sync + 'static {
    type Arg: DeserializeOwned + Type;
    type Result;

    fn exec(&self, ctx: TCtx, input: Self::Arg) -> Result<RequestFuture, ExecError>;

    fn typedef(defs: &mut TypeDefs) -> ProcedureDataType;
}

pub struct DoubleArgMarker<TArg, TResultMarker>(
    /* private */ PhantomData<(TArg, TResultMarker)>,
);
impl<TFunc, TCtx, TArg, TResult, TResultMarker>
    RequestResolver<TCtx, DoubleArgMarker<TArg, TResultMarker>> for TFunc
where
    TArg: DeserializeOwned + Type,
    TFunc: Fn(TCtx, TArg) -> TResult + Send + Sync + 'static,
    TResult: RequestResult<TResultMarker>,
{
    type Result = TResult;
    type Arg = TArg;

    fn exec(&self, ctx: TCtx, input: Self::Arg) -> Result<RequestFuture, ExecError> {
        self(ctx, input).into_request_future()
    }

    fn typedef(defs: &mut TypeDefs) -> ProcedureDataType {
        ProcedureDataType {
            arg_ty: <TArg as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            ),
            result_ty: <TResult::Data as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            ),
        }
    }
}

// pub struct NoArgMarker<TResultMarker>(/* private */ PhantomData<TResultMarker>);
// impl<TFunc, TCtx, TResult, TResultMarker> Resolver<TCtx, NoArgMarker<TResultMarker>> for TFunc
// where
//     TFunc: Fn() -> TResult,
//     TResult: IntoLayerResult<TResultMarker> + Type,
// {
//     fn exec(&self, _ctx: TCtx, _arg: Value) -> Result<LayerResult, ExecError> {
//         self().into_layer_result()
//     }
//
//     fn typedef(defs: &mut TypeDefs) -> ProcedureDataType {
//         ProcedureDataType {
//             arg_ty: <() as Type>::def(DefOpts {
//                 parent_inline: true,
//                 type_map: defs,
//             }),
//             result_ty: <TResult as Type>::def(DefOpts {
//                 parent_inline: true,
//                 type_map: defs,
//             }),
//         }
//     }
// }
//
// pub struct SingleArgMarker<TResultMarker>(/* private */ PhantomData<TResultMarker>);
// impl<TFunc, TCtx, TResult, TResultMarker> Resolver<TCtx, SingleArgMarker<TResultMarker>> for TFunc
// where
//     TFunc: Fn(TCtx) -> TResult,
//     TResult: IntoLayerResult<TResultMarker>,
// {
//     fn exec(&self, ctx: TCtx, _arg: Value) -> Result<LayerResult, ExecError> {
//         self(ctx).into_layer_result()
//     }
//
//     fn typedef(defs: &mut TypeDefs) -> ProcedureDataType {
//         ProcedureDataType {
//             arg_ty: <() as Type>::def(DefOpts {
//                 parent_inline: true,
//                 type_map: defs,
//             }),
//             result_ty: <TResult::Result as Type>::def(DefOpts {
//                 parent_inline: true,
//                 type_map: defs,
//             }),
//         }
//     }
// }

pub trait StreamResolver<TCtx, TMarker> {
    fn exec(&self, ctx: TCtx, input: Value) -> Result<StreamFuture, ExecError>;

    fn typedef(defs: &mut TypeDefs) -> ProcedureDataType;
}

pub struct DoubleArgStreamMarker<TArg, TResult, TStream>(
    /* private */ PhantomData<(TArg, TResult, TStream)>,
);
impl<TFunc, TCtx, TArg, TResult, TStream>
    StreamResolver<TCtx, DoubleArgStreamMarker<TArg, TResult, TStream>> for TFunc
where
    TArg: DeserializeOwned + Type,
    TFunc: Fn(TCtx, TArg) -> TStream,
    TStream: Stream<Item = TResult> + Send + Sync + 'static,
    TResult: Serialize + Type,
{
    fn exec(&self, ctx: TCtx, input: Value) -> Result<StreamFuture, ExecError> {
        let input = serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?;

        Ok(Box::pin(self(ctx, input).map(|v| {
            serde_json::to_value(&v).map_err(ExecError::SerializingResultErr)
        })))
    }

    fn typedef(defs: &mut TypeDefs) -> ProcedureDataType {
        ProcedureDataType {
            arg_ty: <TArg as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            ),
            result_ty: <TResult as Type>::reference(
                DefOpts {
                    parent_inline: false,
                    type_map: defs,
                },
                &[],
            ),
        }
    }
}
