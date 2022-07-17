---
title: Error Handling
layout: ../../layouts/MainLayout.astro
---

**rspc** resolvers are allowed to return a `Result<T, TErr>` where `T` can be any type which can be returned from a normal resolver and `TErr` is any type that implements `Into<rspc::Error>`. This means you can build your own error type and return them from your resolvers.

Using an `rspc::Error`:

```rust
let router = <rspc:::Router>::new()
    .query("ok", |_, args: ()| {
        // The `as` is required due this resolver never returning an `Err` variant and hence Rust is unable to infer the return type.
        Ok("Hello World".into()) as Result<String, rspc::Error>
    })
    .query("err", |_, args: ()| {
        // The `as` is required due this resolver never returning an `Ok` variant and hence Rust is unable to infer the return type.
        Err(Error::new(
            ErrorCode::BadRequest,
            "This is a custom error!".into(),
        )) as Result<String, Error>
    })
    .build();
```

Creating a custom error type:

```rust
pub enum MyCustomError {
    ServerDidABad,
}

impl Into<Error> for MyCustomError {
    fn into(self) -> Error {
        match self {
            MyCustomError::ServerDidABad => {
                Error::new(ErrorCode::InternalServerError, "I am broke".into())
            }
        }
    }
}

let router = <Router>::new()
    .query("returnCustomError", |_, args: ()| {
        // The `as` is required due this resolver never returning an `Ok` variant and hence Rust is unable to infer the return type.
        Err(MyCustomError::ServerDidABad) as Result<String, MyCustomError> 
    })
    .build();
```