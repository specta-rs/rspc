use futures::Stream;
use serde_json::Value;

/// TODO
pub enum OperationKind {
    Query,
    Mutation,
    Subscription,
}

/// TODO: Docs + rename
pub trait OperationTrait {
    type Result;
    const KIND: OperationKind;
}

pub struct Query;

impl OperationTrait for Query {
    type Result = Value;

    const KIND: OperationKind = OperationKind::Query;
}

pub struct Mutation;

impl OperationTrait for Mutation {
    type Result = Value;

    const KIND: OperationKind = OperationKind::Mutation;
}

pub struct Subscription;

impl OperationTrait for Subscription {
    type Result = Box<dyn Stream<Item = Value>>;

    const KIND: OperationKind = OperationKind::Subscription;
}

/// TODO: Docs
/// We are pretending to be an enum so we need to override the Rust naming convention. This is done for a nice external API.
#[allow(non_snake_case, non_upper_case_globals)]
pub mod Operation {
    use super::*;

    pub const Query: Query = Query {};
    pub const Mutation: Mutation = Mutation {};
    pub const Subscription: Subscription = Subscription {};
}
