use rspc::Rspc;

pub const R: Rspc<()> = Rspc::new();

pub mod basic;
pub mod error_handling;
pub mod merging_routers;
pub mod selection;
pub mod subscriptions;
