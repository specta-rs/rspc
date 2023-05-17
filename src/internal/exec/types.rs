mod private {
    use std::borrow::Cow;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// The type of a request to rspc.
    ///
    /// @internal
    #[derive(Deserialize)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(tag = "method", rename_all = "camelCase")]
    pub enum Request {
        Query {
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        Mutation {
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        Subscription {
            /// The ID of the subscription. This is used to identify the subscription.
            /// It is the client's responsibility to ensure that this ID is unique.
            id: Cow<'static, str>,
            path: Cow<'static, str>,
            input: Option<Value>,
        },
        SubscriptionStop {
            id: Cow<'static, str>,
        },
    }

    /// An error that can be returned by rspc.
    ///
    /// @internal
    #[derive(Serialize)]
    #[cfg_attr(test, derive(specta::Type))]
    pub struct ResponseError {
        pub code: i32,
        pub message: String,
        pub data: Option<Value>,
    }

    /// A value that can be a successful result or an error.
    ///
    /// @internal
    #[derive(Serialize)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum ValueOrError {
        /// The result of a successful operation.
        Value(Value),
        /// The result of a failed operation.
        Error(ResponseError),
    }

    /// The type of a response from rspc.
    ///
    /// @internal
    #[derive(Serialize)]
    #[cfg_attr(test, derive(specta::Type))]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum Response {
        // Result of a [Request::Query] or [Request::Mutation].
        Response {
            path: Cow<'static, str>,
            result: ValueOrError,
        },
        // Message emitted by of an active [Request::Subscription].
        Event {
            id: Cow<'static, str>,
            result: ValueOrError,
        },
    }
}

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use private::*;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::*;
