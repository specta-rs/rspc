//! Rust client for [`rspc`].
//!
//! # This is really unstable you should be careful using it!
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

// TODO: Change `exec` to `query`/`mutation`/`subscription` with a bound on the incoming operation?
// TODO: Error handling working (typesafe errors + internal errors being throw not panic)
// TODO: Maybe make inner client a trait so the user can use any HTTP client.
// TODO: Treating `reqwest` as a public or private dependency?
// TODO: Supporting transport formats other than JSON?
// TODO: Is this safe to use from the same app that defines the router? If not we should try and forbid it with a compiler error.

use std::{borrow::Cow, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// TODO
#[derive(Debug, Clone)]
pub struct Client<P> {
    url: Cow<'static, str>,
    client: reqwest::Client,
    phantom: PhantomData<P>,
}

impl<P> Client<P> {
    pub fn new(url: impl Into<Cow<'static, str>>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::builder()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .unwrap(), // TODO: Can this fail?
            phantom: PhantomData,
        }
    }

    pub async fn exec<O: Procedure<Procedures = P>>(
        &self,
        input: O::Input,
    ) -> Result<O::Output, O::Error> {
        let url = format!(
            "{}{}{}",
            self.url,
            if self.url.ends_with("/") { "" } else { "/" },
            O::KEY
        );

        match O::KIND {
            ProcedureKind::Query => {
                let res = self
                    .client
                    .get(&url)
                    .query(&[("input", serde_json::to_string(&input).unwrap())]) // TODO: Error handling
                    .send()
                    .await
                    .unwrap(); // TODO: Error handling

                // TODO: This is just ignoring error handling. This client is designed as a prototype not to be used.
                #[derive(Deserialize)]
                pub struct LegacyFormat {
                    result: LegacyResult,
                }
                #[derive(Deserialize)]
                pub struct LegacyResult {
                    #[serde(rename = "type")]
                    _type: String,
                    data: serde_json::Value,
                }

                let result: LegacyFormat = res.json().await.unwrap();
                Ok(serde_json::from_value(result.result.data).unwrap())
            }
            ProcedureKind::Mutation => {
                let res = self
                    .client
                    .post(&url)
                    .body(serde_json::to_string(&input).unwrap()) // TODO: Error handling
                    .send()
                    .await
                    .unwrap(); // TODO: Error handling

                // TODO: This is just ignoring error handling. This client is designed as a prototype not to be used.
                #[derive(Deserialize)]
                pub struct LegacyFormat {
                    result: LegacyResult,
                }
                #[derive(Deserialize)]
                pub struct LegacyResult {
                    #[serde(rename = "type")]
                    _type: String,
                    data: serde_json::Value,
                }

                let result: LegacyFormat = res.json().await.unwrap();
                Ok(serde_json::from_value(result.result.data).unwrap())
            }
            ProcedureKind::Subscription => {
                // TODO: We will need to implement websocket support somehow. https://github.com/seanmonstar/reqwest/issues/864
                // TODO: Returning a stream
                unimplemented!("subscriptions are not supported yet!");
            }
        }
    }
}

pub trait Procedure {
    type Input: Serialize;
    type Output: DeserializeOwned;
    type Error: DeserializeOwned;
    type Procedures;

    const KEY: &'static str;
    const KIND: ProcedureKind;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureKind {
    Query,
    Mutation,
    Subscription,
}
