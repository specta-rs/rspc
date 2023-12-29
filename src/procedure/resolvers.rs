macro_rules! resolvers {
    ($this:tt, $ctx:ty, $mw_ty:ty, $mw:expr) => {
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, query, QueryOrMutation, Query);
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, mutation, QueryOrMutation, Mutation);
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, subscription, Subscription, Subscription);
    };
    (impl; $this:tt, $ctx:ty, $mw_ty:ty, $mw:expr, $fn_name:ident, $marker:ident, $kind:ident) => {
        pub fn $fn_name<TResolver, TResultMarker, TArg>(
            self,
            resolver: TResolver,
        ) -> Procedure<
            HasResolver<TResolver, TError, $marker<TResultMarker>, crate::internal::resolver::M<TArg>>,
            $mw_ty,
        >
        where
            HasResolver<TResolver, TError, $marker<TResultMarker>, crate::internal::resolver::M<TArg>>: Layer<$ctx>,
            TArg: serde::de::DeserializeOwned + specta::Type,
        {
        	let $this = self;

            let resolver = HasResolver::new(resolver, ProcedureKind::$kind, |type_map| <TArg as specta::Type>::reference(
                type_map,
                &[],
            ));

            // TODO: Make this work
            // // Trade runtime performance for reduced monomorphization
            // #[cfg(debug_assertions)]
            // let resolver = boxed(resolver);

            Procedure::new(resolver, $mw)
        }
    };
}
pub(crate) use resolvers;
