---
title: Error Handling
---

**rspc** resolvers are allowed to return a `Result<T, rspc::Error>` where `T` can be any type which can be returned from a normal resolver.

It is important to understand that the [question mark operation (`?`)](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html) in Rust will expand to `return Err(From::from(err))`. We rely on this fact for custom errors.

### Using an `rspc::Error`:

```rust
let router = <rspc:::Router>::new()
    .query("ok", |_, args: ()| {
        // Rust infers the error type must be `rspc::Error`
        Ok("Hello World".into())
    })
    .query("err", |_, args: ()| {
        // The `as` is required due this resolver never returning an `Ok` variant,
        // hence Rust is unable to infer the Results type.
        Err(Error::new(
            ErrorCode::BadRequest,
            "This is a custom error!".into(),
        )) as Result<String, _ /* Rust can infer the error type */>
    })
    .query("errWithCause", |_, args: ()| {
        Err(Error::with_cause(
            ErrorCode::BadRequest,
            "This is a custom error!".into(),
            // This function must return an error that implements `std::error::Error`
            some_function_returning_error(),
        )) as Result<String, Error>
    })
    .query("errUsingQuestionMarkOperator", |_, args: ()| {
        let value = some_function_returning_an_error()?;
        Ok(value)
    })
    .build();
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/error_handling.rs)

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
    .query("returnCustomErrorUsingQuestionMark", |_, args: ()| {
        Ok(Err(MyCustomError::ServerDidABad)?)
    })
     .query("customErrUsingInto", |_, _args: ()| {
        Err(MyCustomError::IAmBroke.into()) as Result<String, Error>
    })
    .build();
```