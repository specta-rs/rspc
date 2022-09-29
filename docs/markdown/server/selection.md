---
title: Selection
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
            id: 1,
            name: "Monty Beaumont".into(),
            email: "monty@otbeaumont.me".into(),
            age: 7,
            password: "password123".into(),
        };

        selection!(user, { name, age }) // We select only the name and age fields to return
    })
    .query("users", |_, _: ()| {
        let user = User {
            name: "Monty Beaumont".into(),
            email: "monty@otbeaumont.me".into(),
            age: 7,
            password: "password123".into(),
        };

        // We have a vector of data which contains information but we only want to return some of it the user.
        // Eg. We don't want to expose the password field.
        let users = vec![user.clone(), user.clone(), user];
        
        selection!(users, [{ name, age }]) // Here we are selecting the fields we want to expose on each item in the list. This is completely type safe!
    })
    .build();
```

[View full example](https://github.com/oscartbeaumont/rspc/blob/main/examples/selection.rs)