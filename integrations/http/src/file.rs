//! Multipart types by framework:
//!  Axum (multer) - `Multipart::next_field`
//!  Actix Web - `impl Stream<Item = Result<Field, Error>>`
//!  Poem (multer) - `Multipart::next_field`
//!  Warp (multipart) - `impl Stream<Item = Result<Part, Error>>`
//!  Tide - Not supported anymore
//!  Rocket (multer with strong wrapper) - ...
//!  Hyper - Nothing build in. multer or multipart supported.
