use std::marker::PhantomData;

use specta::{reference::Reference, TypeMap};

pub struct QueryOrMutation<M>(PhantomData<M>);
pub struct Subscription<M>(PhantomData<M>);

type ArgTy = fn(&mut TypeMap) -> Reference;
