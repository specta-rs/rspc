//! rspc-axum: Axum integration for [rspc](https://rspc.dev).
use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{on, MethodFilter},
    Router,
};
use httpz::{Endpoint, HttpEndpoint, HttpResponse, Server};

fn method_interop(method: httpz_stuff::http::Method) -> axum::http::Method {
    match method {
        httpz_stuff::http::Method::OPTIONS => axum::http::Method::OPTIONS,
        httpz_stuff::http::Method::GET => axum::http::Method::GET,
        httpz_stuff::http::Method::POST => axum::http::Method::POST,
        httpz_stuff::http::Method::PUT => axum::http::Method::PUT,
        httpz_stuff::http::Method::DELETE => axum::http::Method::DELETE,
        httpz_stuff::http::Method::HEAD => axum::http::Method::HEAD,
        httpz_stuff::http::Method::TRACE => axum::http::Method::TRACE,
        httpz_stuff::http::Method::CONNECT => axum::http::Method::CONNECT,
        httpz_stuff::http::Method::PATCH => axum::http::Method::PATCH,
        _ => unreachable!(),
    }
}

fn method_interop2(method: axum::http::Method) -> httpz_stuff::http::Method {
    match method {
        axum::http::Method::OPTIONS => httpz_stuff::http::Method::OPTIONS,
        axum::http::Method::GET => httpz_stuff::http::Method::GET,
        axum::http::Method::POST => httpz_stuff::http::Method::POST,
        axum::http::Method::PUT => httpz_stuff::http::Method::PUT,
        axum::http::Method::DELETE => httpz_stuff::http::Method::DELETE,
        axum::http::Method::HEAD => httpz_stuff::http::Method::HEAD,
        axum::http::Method::TRACE => httpz_stuff::http::Method::TRACE,
        axum::http::Method::CONNECT => httpz_stuff::http::Method::CONNECT,
        axum::http::Method::PATCH => httpz_stuff::http::Method::PATCH,
        _ => unreachable!(),
    }
}

fn status_interop(status: httpz_stuff::http::StatusCode) -> axum::http::StatusCode {
    axum::http::StatusCode::from_u16(status.as_u16()).expect("unreachable")
}

fn headers_interop(headers: httpz_stuff::http::HeaderMap) -> axum::http::HeaderMap {
    let mut new_headers = axum::http::HeaderMap::new();
    for (key, value) in headers.iter() {
        new_headers.insert(
            axum::http::HeaderName::from_bytes(key.as_str().as_bytes()).expect("unreachable"),
            axum::http::HeaderValue::from_bytes(value.as_bytes()).expect("unreachable"),
        );
    }
    new_headers
}

fn headers_interop2(headers: axum::http::HeaderMap) -> httpz_stuff::http::HeaderMap {
    let mut new_headers = httpz_stuff::http::HeaderMap::new();
    for (key, value) in headers.iter() {
        new_headers.insert(
            httpz_stuff::http::HeaderName::from_bytes(key.as_str().as_bytes())
                .expect("unreachable"),
            httpz_stuff::http::HeaderValue::from_bytes(value.as_bytes()).expect("unreachable"),
        );
    }
    new_headers
}

pub fn endpoint<S>(mut endpoint: Endpoint<impl HttpEndpoint>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let (url, methods) = endpoint.endpoint.register();
    let endpoint = Arc::new(endpoint.endpoint);

    let mut method_filter = None::<MethodFilter>;
    for method in methods.as_ref().iter() {
        #[allow(clippy::unwrap_used)] // TODO: Error handling
        let new_filter = MethodFilter::try_from(method_interop(method.clone())).unwrap();
        if let Some(filter) = method_filter {
            method_filter = Some(filter.or(new_filter));
        } else {
            method_filter = Some(new_filter);
        }
    }

    Router::<S>::new().route(
        url.as_ref(),
        on(
            method_filter.expect("no methods specified"), // Unreachable because rspc specifies at least one method
            |state: State<S>, request: axum::extract::Request<Body>| async move {
                let (mut parts, body) = request.into_parts();
                parts.extensions.insert(state);

                // TODO: Should probs limit the size of the body
                let body = match to_bytes(body, usize::MAX).await {
                    Ok(body) => body.to_vec(),
                    Err(err) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            HeaderMap::new(),
                            err.to_string().as_bytes().to_vec(),
                        );
                    }
                };

                let (mut new_parts, _) = httpz_stuff::http::Request::new(()).into_parts();
                new_parts.method = method_interop2(parts.method);
                new_parts.uri =
                    httpz_stuff::http::Uri::try_from(parts.uri.to_string()).expect("unreachable");
                new_parts.version = match parts.version {
                    axum::http::Version::HTTP_10 => httpz_stuff::http::Version::HTTP_10,
                    axum::http::Version::HTTP_11 => httpz_stuff::http::Version::HTTP_11,
                    axum::http::Version::HTTP_2 => httpz_stuff::http::Version::HTTP_2,
                    axum::http::Version::HTTP_3 => httpz_stuff::http::Version::HTTP_3,
                    _ => unreachable!(),
                };
                new_parts.headers = headers_interop2(parts.headers);
                // new_parts.extensions.extend(parts.extensions.clone());

                match endpoint
                    .handler(httpz_stuff::Request::new(
                        httpz_stuff::http::Request::from_parts(new_parts, body),
                        Server::Axum,
                    ))
                    .await
                    .into_response()
                {
                    Ok(resp) => {
                        let (parts, body) = resp.into_parts();
                        (
                            status_interop(parts.status),
                            headers_interop(parts.headers),
                            body,
                        )
                    }
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        HeaderMap::new(),
                        err.to_string().as_bytes().to_vec(),
                    ),
                }
            },
        ),
    )
}
