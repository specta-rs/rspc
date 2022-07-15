---
title: Selection
layout: ../../layouts/MainLayout.astro
---

It is very common when building an API to fetch some data from the database but you only want to expose a subset of the data to the client. With rspc you can use the `selection!` macro to easily return a subset of fields on a struct.

For example say you have a `User` struct like the following:

```rust
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub password: String,
}
```

If your database returns a `User` struct you are unable to return it directly from your resolver as that would leak the value in the `password` field. Traditionally you would have to create a second struct without the `password` field, however this isn't optimal as it adds unnecessary boilerplate to your project. Instead you can use the `selection!` macro like below to select only certain fields from the struct.

```rust
let router = <Router>::new()
    .query("me", |_, _: ()| {
        // This struct would be returned from your database!
        let user = User {
            name: "Monty Beaumont".into(),
            email: "monty@otbeaumont.me".into(),
            age: 7,
            password: "password123".into(),
        };

        selection!(user, { name, age }) // We select only the name and age fields to return
    })
    .build();
```