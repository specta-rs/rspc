---
title: Error Handling
index: 22
---

rspc procedures have to return the type `Result<T, rspc::Error>` where `T` can be any type which can be returned from a normal procedure.

The fact that Rust as a language currently requires the error type to be concrete makes error handling slightly annoying. All of the error handling done by rspc relys on the [question mark operator (`?`)](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html) in Rust to make a reasonable developer experience. The question mark operator will expand into something along the lines of `return Err(From::from(err))` under the hood. This means for any type `T` if you implement `From<T> for rspc::Error` you will be able to rely on the question mark operator to convert it into an `rspc::Error` type.

### An example using the `rspc::Error` type

```rust
use rspc::{Error, Router};

let router = <Router>::new()
    .query("ok", |t| {
        t(|_, args: ()| {
            // Rust infers the return type is `Result<String, rspc::Error>`
            Ok("Hello World".into())
        })
    })
    .query("err", |t| {
        t(|_, args: ()| {
            // Rust is unable to infer the `Ok` variant of the result.
            // We use the `as` keyword to tell Rust the type of the result.
            // This situation is rare in real world code.
            Err(Error::new(
                ErrorCode::BadRequest,
                "This is a custom error!".into(),
            )) as Result<String, _ /* Rust can infer the error type */>
        })
    })
    .query("errWithCause", |t| {
        t(|_, args: ()| {
            some_function_returning_error().map_err(|err| {
                Error::with_cause(
                    ErrorCode::BadRequest,
                    "This is a custom error!".into(),
                    // This error type implements `std::error::Error`
                    err,
                )
            })
        })
    })
    .build();
```

### Custom error type

```rust
pub enum MyCustomError {
    ServerDidABad,
}

impl From<MyCustomError> for rspc::Error {
    fn from(_: MyCustomError) -> Self {
        rspc::Error::new(rspc::ErrorCode::InternalServerError, "Server did an oopsie".into())
    }
}

let router = <Router>::new()
    .query("returnCustomErrorUsingQuestionMark", |t| {
        t(|_, args: ()| Ok(Err(MyCustomError::ServerDidABad)?))
    })
    .query("customErrUsingInto", |t| {
        t(|_, _args: ()| {
            let res: Result<String, MyCustomError> = some_function();
            res.map_err(Into::into) // This is an alternative to using the question mark operator
        })
    })
    .build();
```