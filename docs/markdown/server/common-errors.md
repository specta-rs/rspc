---
title: Common errors
---

rspc uses traits to allow for any nearly any type to be returned from your procedures, however this can make the error messages hard to understand so some guidance is provided here.

#### the trait `IntoLayerResult<_>` is not implemented for type

This error means the type which you returned from your procedure is not valid. This is probably because it doesn't implement the traits:

 - [`serde::Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html)
 - [`specta::Type`](https://docs.rs/specta/latest/specta/trait.Type.html)

To fix this error ensure the custom types which you return from your procedure have the derive macros as shown below or that the type is a [Rust primitive type](https://doc.rust-lang.org/book/ch03-02-data-types.html).

```rust
use rspc::Type;
use serde::Serialize; // This requires the 'derive' feature to be enabled.

#[derive(Type, Serialize)]
struct MyStruct {}

#[derive(Type, Serialize)]
enum MyStruct {
    SomeVariant
}

// Type aliases do not require the derive macro.
type AnotherName = MyStruct;
```

If you are unable to determine what is causing your type to be invalid you can use the following utility functions to get a better warning from the Rust compiler. Ensure you don't keep this utility function in production code.

```rust
pub struct Demo {}

rspc::test_result_type::<Demo>();
rspc::test_result_value(Demo {});
```

#### the trait `for<'de> serde::de::Deserialize<'de>` is not implemented for

This error means that the type which you specified as your argument type is invalid. Ensure the type implements the trait [`serde::DeserializeOwned`](https://docs.rs/serde/latest/serde/de/trait.DeserializeOwned.html).

This can be done using the `Deserialize` derive macro provided by [serde](https://serde.rs/derive.html):

```rust
use serde::Deserialize; // This requires the 'derive' feature to be enabled.

#[derive(Deserialize)]
struct MyStruct {}

#[derive(Deserialize)]
enum MyStruct {
    SomeVariant
}

// Type aliases do not require the derive macro.
type AnotherName = MyStruct;
```

#### the trait `Type` is not implemented for

This error means that the type which you specified as your argument type is invalid. Ensure the type implements the trait [`specta::Type`](https://docs.rs/specta/latest/specta/trait.Type.html).

This can be done using the `Type` derive macro:

```rust
use rspc::Type;

#[derive(Type)]
struct MyStruct {}

#[derive(Type)]
enum MyStruct {
    SomeVariant
}
```

#### type mismatch in closure arguments

This is probably caused by you incorrectly hardcoding the type for the request context (first argument) of the procedure closure.

```rust
// INVALID CODE
Router::<()>::new() // Here we set the context to `()` but we set the closures argument type to `i32`.
    .query("debug", |t| t(|ctx: i32, _: ()| {}))

// SOLUTION
Router::<()>::new() // Here we don't set the type of the context on the closure and Rust infers it.
    .query("debug", |t| t(|ctx, _: ()| {}))
```
