// use rspc::internal::middleware::MiddlewareContext;

// fn todo() {
//     mw(|mw, ctx| async move { mw.next(ctx) });

//     // mw(|mw, ctx| async move { mw.next(ctx).into_future().await });
// }

// pub trait Middleware<TLCtx>: Fn(MiddlewareContext, TLCtx) -> Self::Result {
//     type Result: MiddlewareResult<TLCtx>;
// }

// impl<F: Fn(MiddlewareContext, TLCtx)> Middleware for F {}

// pub trait MiddlewareResult<TLCtx> {
//     type Result: Send + 'static;
// }

// impl<Fu: Future<Output = ()>> MiddlewareResult<TLCtx> for Fu {}

// impl<M: ArgMapper, R:MiddlewareResult<TLCtx> F: Fn(MiddlewareContext, TLCtx) -> R> MiddlewareResult<TLCtx> for ArgMapper<M, Fu>  {}
