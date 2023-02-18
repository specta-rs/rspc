---
title: Specta
---

For rspc to be able to convert your types into Typescript they must implement the `specta::Type` trait. [Specta](https://github.com/oscartbeaumont/specta) is a crate that was created so that rspc can introspect Rust types. The `Type` trait allows the Typescript exporter to understand the fields, generics and dependant types of a Rust type.

The easiest way to implement the `specta::Type` trait is by using the `rspc::Type` derive macro. We have already implemented most in-built types if you can find a missing one open a [GitHub Issue](https://github.com/oscartbeaumont/rspc).

```rust
use rspc::Type;

#[derive(Type)]
pub struct MyStruct {
    pub name: String,
    pub age: i32,
}

#[derive(Type)]
pub enum MyEnum {
    SomeVariant,
    // It is import MyStruct also implements `Type` or this will not work
    AnotherVariant(MyStruct),
}
```

### Limitations

You should be careful when using generics with [type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html) as you may run into situations where the types are not exported correctly. As far as we are aware this is a known limitation with Rust and is not something we can fix. If you run into this edge case you should change to using a `struct` instead of a type alias to workaround the problem.

### Other languages

Specta stores information about your type which means an exporter for languages other than [Typescript](https://www.typescriptlang.org) could be made. If you are interested in supporting other languages, you can do so directly using the `specta::Type` trait in your own project, however a pull request to Specta would be appreciated.

### Specta without rspc

Specta is an independent crate and can be used without rspc. Refer to it's [documentation](https://docs.rs/specta) for support using it.
