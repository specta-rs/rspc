use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use serde_json::Value;

use crate::{ConcreteArg, ExecError, Key, KeyDefinition, Operation};

/// TODO
pub struct CompiledRouter<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub(crate) query: Operation<TQueryKey, TCtx>,
    pub(crate) mutation: Operation<TMutationKey, TCtx>,
    pub(crate) subscription: Operation<TSubscriptionKey, TCtx>,
    pub(crate) phantom: PhantomData<TMeta>,
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    CompiledRouter<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub async fn exec_query<TArg, TKey>(
        &self,
        ctx: TCtx,
        key: TKey,
        arg: TArg,
    ) -> Result<Value, ExecError>
    where
        TArg: Send + Sync + 'static,
        TKey: Key<TQueryKey, TArg>,
    {
        let definition = self
            .query
            .get(key.to_val())
            .ok_or(ExecError::OperationNotFound)?;
        let arg = match TypeId::of::<TArg>() == TypeId::of::<Value>() {
            // SAFETY: We check the TypeId's match before `transmute_copy`. We use this method as I couldn't implement a trait which wouldn't overlap to abstract this into.
            true => {
                // We are using runtime specialization because I could not come up with a trait which wouldn't overlap to abstract this into.
                let v = (&mut Some(arg) as &mut dyn Any)
                    .downcast_mut::<Option<Value>>()
                    .unwrap()
                    .take()
                    .unwrap();
                ConcreteArg::Value(v)
            }
            false => ConcreteArg::Unknown(Box::new(arg)),
        };

        definition(ctx, arg)?.await
    }

    #[allow(dead_code)]
    pub(crate) async fn exec_query_unsafe(
        &self,
        ctx: TCtx,
        key: String,
        arg: Value,
    ) -> Result<Value, ExecError> {
        let definition = self
            .query
            .get(TQueryKey::from_str(key)?)
            .ok_or(ExecError::OperationNotFound)?;
        definition(ctx, ConcreteArg::Value(arg))?.await
    }

    pub async fn exec_mutation<TArg, TKey>(
        &self,
        ctx: TCtx,
        key: TKey,
        arg: TArg,
    ) -> Result<Value, ExecError>
    where
        TArg: Send + Sync + 'static,
        TKey: Key<TMutationKey, TArg>,
    {
        let definition = self
            .mutation
            .get(key.to_val())
            .ok_or(ExecError::OperationNotFound)?;
        let arg = match TypeId::of::<TArg>() == TypeId::of::<Value>() {
            true => {
                // We are using runtime specialization because I could not come up with a trait which wouldn't overlap to abstract this into.
                let v = (&mut Some(arg) as &mut dyn Any)
                    .downcast_mut::<Option<Value>>()
                    .unwrap()
                    .take()
                    .unwrap();
                ConcreteArg::Value(v)
            }
            false => ConcreteArg::Unknown(Box::new(arg)),
        };

        definition(ctx, arg)?.await
    }

    #[allow(dead_code)]
    pub(crate) async fn exec_mutation_unsafe(
        &self,
        ctx: TCtx,
        key: String,
        arg: Value,
    ) -> Result<Value, ExecError> {
        let definition = self
            .mutation
            .get(TMutationKey::from_str(key)?)
            .ok_or(ExecError::OperationNotFound)?;
        definition(ctx, ConcreteArg::Value(arg))?.await
    }
}
