---
title: Concepts
index: 21
---

# Capturing variables

rspc allows for capturing variables in the closure of a procedure. This is generally fround upon as it put a requirement on that value when creating the router which could limit your ability to unit test the router. More of the logic behind this is explained in request context section below. This is a general rule and you will likely find exceptions.

```rust

// NOT-RECOMMEND - Capturing variables
// You should avoid providing having arguments to your mount function
pub(crate) fn mount(db: DatabaseConn) -> Router {
    // The `move` on the next line is the best indication that you are capturing variables.
    <Router>::new().query("getUsers", move |t| {
        t(move |_, _: ()| async move { db.users().find_all().exec().await })
    });
}


// RECOMMEND - Using Request Context
struct MyCtx { db: DatabaseConn }

pub(crate) fn mount() -> Router {
    Router::<MyCtx>::new().query("getUsers", |t| {
        t(|ctx: MyCtx, _: ()| async move { ctx.db.users().find_all().exec().await })
    });
}

```

# Request Context

When calling execute on a operation you must provide a request context. The type of the request context must match the `TCtx` generic parameter defined on the `rspc::Router`.

Using request context is important because it means you can construct the router without a dependency on anything (such a database) which allows you to validate the router in a unit test. The routes are stringly typed so we can't just rely on Rust's compiler to validate the router. This tradeoff was made for the superior developer experience as we believe using request context and a unit test for validating the router is able to mitigate the risk.

A request context is created on every request and can hold any data the user wants. The request context also abstracts the underlying transport layer such as HTTP, Websocket or Tauri so that the router can be agonistic to which one is being used.

```rust
struct MyCtx {
    db: Arc<Database>,
    some_value: &'static str
}

// Axum shown here as an example. This could be any transport.
fn main() {
    let db = Arc::new(Database::new());

    // Setup your rspc router to take your custom context type
    let router = Router::<MyCtx>::new()
        .query("myQuery", |t| t(|ctx, input: ()| {
            assert_eq!(ctx.some_value, "Hello World");
        }))
        .build();

    axum::Router::new()
        // Attach the rspc router to your axum router
        // The closure you provide is used to create a new request context for each request
        .route("/rspc/:id",
            router
                .endpoint(move || MyCtx {
                    db: db.clone(),
                    some_value: "Hello World",
                })
                .axum()
        )
}
```