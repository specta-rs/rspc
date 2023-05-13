use std::mem;

use httpz::{axum::axum::extract::FromRequestParts, http};

use super::CookieJar;

/// TODO
///
/// This wraps [httpz::Request] removing any methods that are not safe with rspc such as `body`, `into_parts` and replacing the cookie handling API.
///
#[derive(Debug)]
pub struct Request(httpz::Request, Option<CookieJar>);

impl Request {
    pub(crate) fn new(req: httpz::Request, cookies: Option<CookieJar>) -> Self {
        Self(req, cookies)
    }

    /// Get the uri of the request.
    pub fn uri(&self) -> &httpz::http::Uri {
        self.0.uri()
    }

    /// Get the version of the request.
    pub fn version(&self) -> httpz::http::Version {
        self.0.version()
    }

    /// Get the method of the request.
    pub fn method(&self) -> &httpz::http::Method {
        self.0.method()
    }

    /// Get the headers of the request.
    pub fn headers(&self) -> &httpz::http::HeaderMap {
        self.0.headers()
    }

    /// Get the headers of the request.
    pub fn headers_mut(&mut self) -> &mut httpz::http::HeaderMap {
        self.0.headers_mut()
    }

    /// TODO
    pub fn cookies(&mut self) -> Option<CookieJar> {
        // TODO: This take means a `None` response could be because it was already used or because it's a websocket. This is a confusing DX and needs fixing.

        mem::replace(&mut self.1, None)
    }

    /// query_pairs returns an iterator of the query parameters.
    pub fn query_pairs(&self) -> Option<httpz::form_urlencoded::Parse<'_>> {
        self.0.query_pairs()
    }

    /// TODO
    pub fn server(&self) -> httpz::Server {
        self.0.server()
    }

    /// Get the extensions of the request.
    pub fn extensions(&self) -> &http::Extensions {
        self.0.extensions()
    }

    /// Get the extensions of the request.
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        self.0.extensions_mut()
    }

    /// This methods allows using Axum extractors.
    /// This was previously supported but in Axum 0.6 it's not typesafe anymore so we are going to remove this API.
    // TODO: Remove this API once rspc's official cookie API is more stabilised.
    #[cfg(feature = "axum")]
    pub fn deprecated_extract<E, S>(&mut self) -> Option<Result<E, E::Rejection>>
    where
        E: FromRequestParts<S>,
        S: Clone + Send + Sync + 'static,
    {
        let parts = self.0.parts_mut();

        let state = parts
            .extensions
            .remove::<httpz::axum::axum::extract::State<S>>()?;

        // This is bad but it's a temporary API so I don't care.
        Some(futures::executor::block_on(async {
            let resp = <E as FromRequestParts<S>>::from_request_parts(parts, &state.0).await;
            parts.extensions.insert(state);
            resp
        }))
    }
}
