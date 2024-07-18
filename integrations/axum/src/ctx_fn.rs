use axum::{extract::FromRequestParts, http::request::Parts};
use std::{future::Future, marker::PhantomData};

// TODO: Sealed?
pub trait ContextFunction<TCtx, TState, TMarker>: Clone + Send + Sync + 'static
where
    TState: Send + Sync,
    TCtx: Send + 'static,
{
    // fn exec(&self, parts: Parts, state: &TState) -> impl Future<Output = Result<TCtx, ()>> + Send;
}

pub struct ZeroArgMarker;
impl<TCtx, TFunc, TState> ContextFunction<TCtx, TState, ZeroArgMarker> for TFunc
where
    TFunc: Fn() -> TCtx + Clone + Send + Sync + 'static,
    TState: Send + Sync,
    TCtx: Send + 'static,
{
    // async fn exec(&self, _: Parts, _: &TState) -> Result<TCtx, ()> {
    //     Ok(self.clone()())
    // }
}

macro_rules! impl_fn {
    ($marker:ident; $($generics:ident),*) => {
    		#[allow(unused_parens)]
        pub struct $marker<$($generics),*>(PhantomData<($($generics),*)>);

        impl<TCtx, TFunc, TState, $($generics: FromRequestParts<TState> + Send),*> ContextFunction<TCtx, TState, $marker<$($generics),*>> for TFunc
        where
            TFunc: Fn($($generics),*) -> TCtx + Clone + Send + Sync + 'static,
            TState: Send + Sync,
            TCtx: Send + 'static
        {
            // async fn exec(&self, mut parts: Parts, state: &TState) -> Result<TCtx, ExecError>
            // {
		    //         $(
			// 							#[allow(non_snake_case)]
			// 							let Ok($generics) = $generics::from_request_parts(&mut parts, &state).await else {
			// 									 return Err(ExecError::AxumExtractorError)
			// 							};
			// 					)*

            //     Ok(self.clone()($($generics),*))
            // }
        }
    };
}

impl_fn!(OneArgMarker; T1);
impl_fn!(TwoArgMarker; T1, T2);
impl_fn!(ThreeArgMarker; T1, T2, T3);
impl_fn!(FourArgMarker; T1, T2, T3, T4);
impl_fn!(FiveArgMarker; T1, T2, T3, T4, T5);
impl_fn!(SixArgMarker; T1, T2, T3, T4, T5, T6);
impl_fn!(SevenArgMarker; T1, T2, T3, T4, T5, T6, T7);
impl_fn!(EightArgMarker; T1, T2, T3, T4, T5, T6, T7, T8);
impl_fn!(NineArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_fn!(TenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_fn!(ElevenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_fn!(TwelveArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_fn!(ThirteenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_fn!(FourteenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_fn!(FifteenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_fn!(SixteenArgMarker; T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
