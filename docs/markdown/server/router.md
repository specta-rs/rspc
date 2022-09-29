---
title: Router
index: 20
---

A router contains a collection of procedures (queries, mutations or subscriptions) that can be called by a client. A router has many generic arguments which can be configured by the user to match the type of data that the router will be handling. A router is defined as `Router<TCtx, TMeta, TMiddleware>`.

# TCtx

The type of the [request context](/server/concepts#request-context). This is usually a `struct` containing state coming from the webserver such as a database connection and user session.

For example:

```rust
pub struct MyCtx {
    // The database connection. Prisma Client Rust shown here.
    db: Arc<PrismaClient>,

    // The session_id which could be extracted from HTTP cookies.
    session_id: Option<String>,

    // The HTTP cookie jar. The `Cookies` type is hypothetical.
    cookies: Cookies, 

    // An `Arc` allows us to hold multiple immutable references to the message.
    // The `Mutex` allows interior mutability so we can safely modify the string from a immutable reference.
    my_cool_msg: Arc<Mutex<String>>,
}
```

Constructing an rspc router with a specific context type is done as following.

```rust
// Create a router with the default `TCtx` of `()`
let router = <Router>::new();

// Create a router with a custom `TCtx` type
let router = Router::<MyCtx>::new();
````

`TCtx` is super powerful in rspc because [middleware](/server/middleware) are able to change it for procedures following them in the chain. A great example of this in action is an authentication middleware like shown below.

```rust
pub struct AuthenticatedCtx {
    db: Arc<PrismaClient>, // Your database connection
    user: User, // Your user model in Rust.
}

// We define a router with the `TCtx` type set to `MyCtx`
let router = Router::<MyCtx>::new()
    // We then define the version query before the middleware so that it doesn't require authentication.
    .query("version", |t| {
        t(|ctx: MyCtx, _: ()| "1.0.0")
    })
    // Then we define a middleware which is responsible for rejecting unauthorized requests.
    .middleware(|mw| mw.middleware(|mw| async move {
        let old_ctx = mw.ctx;
        match old_ctx.session_id {
            Some(ref session_id) => {
                // .with_ctx changes the type of `TCtx` for all preceding procedures.
                Ok(mw.with_ctx(AuthenticatedCtx {
                    user: User::from_session(session_id).await?
                }))
            }
            None => Err(rspc::Error::new(
                ErrorCode::Unauthorized,
                "Unauthorized".into(),
            )),
        }
    }))
     // We then define a query to return the current user. This will only be called for authenticated users.
    .query("getMe", |t| {
        // See how we now take in `AuthenticatedCtx` with all the data from the middleware
        t(|ctx: AuthenticatedCtx, _: ()| ctx.user)
    })
    .build();
```

# TMeta

For all intents and purposes keep this `()`. This generic argument does nothing in the current release and may be deprecated in the future.

# TMiddleware

This argument holds the instance of the last middleware builder which you mounted onto your router through a `.middleware(...)` call. You generally don't need to worry about this generic but is what allows the context switching to work.

# Attaching procedures

Procedures represent a function you define in Rust which can be called from the frontend. You can define them on your routing like the following example.

```rust
let router = <Router>::new()
    // Define a query taking no arguments and returning "1.0.0"
    .query("version", |t| t(|ctx, input: ()| "1.0.0"))
    // Define a query taking a string and returning it
    .query("echo", |t| t(|ctx, input: String| input))
    // Define a query which does an asynchronous operation.
    .query("getUsers", |t| t(|ctx, input: String| async move {
        await User::get_all() // returns `User`
    }))

    // The same syntax as above can be used for mutations.
    .mutation("createUser", |t| t(|ctx, new_user: User| async move {
        await new_user.create() // Returns `()`
    }))

    // Subscriptions can also be used for server -> client real time events
    // Subscriptions have a slightly different syntax. You can respond with any Rust `Stream` type.
    .subscription("pings", |t| t(|ctx, input: ()| async_stream::stream! {
        for i in 0..5 {
            yield "ping".to_string();
            sleep(Duration::from_secs(1)).await;
        }
    }))
    .build(); // Ensure you build once you have added all your operations.
```

# Should I use a query or a mutation?

Does your operation have **side effects**? If so, use a mutation else, use a query.

A query should not change any data on the server, it should just be responsible for fetching data. A mutation should be responsible for changing data on the server.

# Merging routers

When building an API server, you will often want to split up your endpoints into multiple files to make the code easier to work on. You can combine routers using the `.merge` method.

`router.merge(prefix: &'static str, router: Router)`

```rust
// This could be defined in another file or even another crate
let users_router = <Router>::new()
        .query("list", |t| t(|ctx, input: ()| vec![] as Vec<()>));

let router = <Router>::new()
    .query("version", |t| t(|_ctx, _: ()| "1.0.0"))
    // The first parameter is a prefix to add to all routes in the merged router.
    .merge("users.", users_router) // You can now call `users.list` from your frontend.
    .build();
```

# Invalidate query

🚧 WIP - [Tracking issue #19](https://github.com/oscartbeaumont/rspc/issues/19)

# Method chaining

When combining multiple operations, you must ensure you chain the method calls or shadow the router variable. This is required due to the way the generics work on the Router.

```rust
// Chaining method calls
let router = <Router>::new()
    .query("version", |t| t(|ctx, input: ()| todo!()))
    .mutation("createUser", |t| t(|ctx, input: ()| todo!()))
    .build();

// Shadowing variable
let router = <Router>::new()
    .query("version", |t| t(|ctx, input: ()| todo!()))
    .mutation("createUser", |t| t(|ctx, input: ()| todo!()));
let router = router
    .mutation("deleteUser", |t| t(|ctx, input: ()| todo!()));
let router = router.build();
```

# Exporting the Typescript bindings

There are two methods to export the Typescript bindings. You can either use the `export_ts_bindings` configuration option or call the `export_ts` function directly on the build router.

```rust
let router = <Router>::new()
    .config(
        Config::new()
            // Doing this will automatically export the bindings when the `build` function is called.
            .export_ts_bindings(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts"))
    )
    .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
    .build();

// Doing it this way you have the flexibility to export it at any time and to wheerever you want.
router.export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("./bindings.ts")).unwrap();
```

You can also use the `set_ts_bindings_header` option on the `Config` if you want to add a custom header to the top of the generated file. This is useful to disable [ESLint](https://eslint.org), [Prettier](https://prettier.io) or other similar tools from processing the generated file.

```rust
let router = <Router>::new()
    .config(
        Config::new()
            // This text is added to the start of the exported Typescript file.
            .set_ts_bindings_header("/* eslint-disable */")
    ))
    .query("version", |t| t(|_, _: ()| env!("CARGO_PKG_VERSION")))
    .build();
```
