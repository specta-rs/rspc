//! Authorization library for rspc.
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

use std::{
    borrow::Cow,
    fmt,
    marker::PhantomData,
    str,
    sync::{Arc, Mutex, PoisonError},
};

use cookie::Cookie;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, ser::SerializeStruct, Serialize};
use specta::Type;

type ResponseCookie = Arc<Mutex<Option<String>>>;

pub struct ZerResponse {
    cookies: ResponseCookie,
}

impl ZerResponse {
    pub fn set_cookie_header(&self) -> Option<String> {
        self.cookies
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub struct Zer<S> {
    cookie_name: Cow<'static, str>,
    key: EncodingKey,
    key2: DecodingKey,
    cookies: Vec<Cookie<'static>>,
    resp_cookies: ResponseCookie,
    phantom: PhantomData<S>,
}

impl<S> Clone for Zer<S> {
    fn clone(&self) -> Self {
        // TODO: Should we `Arc` some stuff?
        Self {
            cookie_name: self.cookie_name.clone(),
            key: self.key.clone(),
            key2: self.key2.clone(),
            cookies: self.cookies.clone(),
            resp_cookies: self.resp_cookies.clone(),
            phantom: PhantomData,
        }
    }
}

impl<S> fmt::Debug for Zer<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Zer").finish()
    }
}

impl<S: Serialize + DeserializeOwned> Zer<S> {
    pub fn from_request(
        cookie_name: impl Into<Cow<'static, str>>,
        secret: &[u8],
        cookie: Option<impl AsRef<[u8]>>,
    ) -> (Self, ZerResponse) {
        let mut cookies = vec![];
        if let Some(cookie) = cookie {
            // TODO: Error handling
            for cookie in Cookie::split_parse_encoded(str::from_utf8(&cookie.as_ref()).unwrap()) {
                cookies.push(cookie.unwrap().into_owned()); // TODO: Error handling
            }
        }

        let resp_cookies = ResponseCookie::default();

        (
            Self {
                cookie_name: cookie_name.into(),
                key: EncodingKey::from_secret(secret),
                key2: DecodingKey::from_secret(secret),
                cookies,
                resp_cookies: resp_cookies.clone(),
                phantom: PhantomData,
            },
            ZerResponse {
                cookies: resp_cookies,
            },
        )
    }

    pub fn session(&self) -> Result<S, UnauthorizedError> {
        self.cookies
            .iter()
            .find(|cookie| cookie.name() == self.cookie_name)
            .map(|cookie| {
                let token = cookie.value();
                let mut v = Validation::new(Algorithm::HS256);
                v.required_spec_claims = Default::default(); // TODO: This is very insecure!

                // TODO: error handling (maybe move this whole thing into `from_request`)
                decode::<S>(token, &self.key2, &v)
                    .unwrap()
                    // TODO: Expose the header somehow?
                    .claims
            })
            .ok_or(UnauthorizedError)
    }

    // TOOD: Ensure calls to `Self::session` return the new session within the same request
    pub fn set_session(&self, session: &S) {
        let token = encode(&Header::default(), &session, &self.key).unwrap();

        self.resp_cookies
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            // TODO: Don't `Clone` the cookie name
            .replace(Cookie::new(self.cookie_name.clone(), token).to_string());
    }
}

#[derive(Debug, Clone)]
pub struct UnauthorizedError;

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unauthorized")
    }
}

impl std::error::Error for UnauthorizedError {}

impl Serialize for UnauthorizedError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Unauthorized", 1)?;
        s.serialize_field("error", "Unauthorized")?;
        s.end()
    }
}

impl Type for UnauthorizedError {
    fn inline(
        _type_map: &mut specta::TypeCollection,
        _generics: specta::Generics,
    ) -> specta::datatype::DataType {
        specta::datatype::DataType::Primitive(specta::datatype::PrimitiveType::String)
    }
}
