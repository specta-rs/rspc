//! rspc-cache: Caching middleware for rspc
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

// TODO: Built-in TTL cache
// TODO: Allow defining custom cache lifetimes (copy Next.js cacheLife maybe)
// TODO: Allow defining a remote cache (e.g. Redis)
