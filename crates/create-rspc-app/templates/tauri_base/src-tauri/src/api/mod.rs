pub use rspc::RouterBuilder;

#[derive(Clone)]
pub struct Ctx {}

pub type Router = rspc::Router<Ctx>;

pub(crate) fn new() -> RouterBuilder<Ctx> {
    Router::new().query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
}
