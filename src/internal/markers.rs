mod private {
    use std::marker::PhantomData;

    use crate::internal::{jsonrpc::RequestKind, ProcedureKind};

    // TODO: I don't wanna call these markers cause they are runtime not just type level. Rename them.

    // TODO: Rename
    pub trait NotAMarker: 'static {
        fn kind(&self) -> ProcedureKind;
    }

    pub struct RequestLayerMarker<T>(RequestKind, PhantomData<T>);

    impl<T> RequestLayerMarker<T> {
        pub(crate) fn new(kind: RequestKind) -> Self {
            Self(kind, Default::default())
        }
    }

    impl<T: 'static> NotAMarker for RequestLayerMarker<T> {
        fn kind(&self) -> ProcedureKind {
            match self.0 {
                RequestKind::Query => ProcedureKind::Query,
                RequestKind::Mutation => ProcedureKind::Mutation,
            }
        }
    }

    pub struct StreamLayerMarker<T>(PhantomData<T>);

    impl<T> StreamLayerMarker<T> {
        pub(crate) fn new() -> Self {
            Self(Default::default())
        }
    }

    impl<T: 'static> NotAMarker for StreamLayerMarker<T> {
        fn kind(&self) -> ProcedureKind {
            ProcedureKind::Subscription
        }
    }
}

pub(crate) use private::{NotAMarker, RequestLayerMarker, StreamLayerMarker};
