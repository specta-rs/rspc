---
title: Middleware
index: 23
---

rspc allows adding middleware to your router which can intercept the request and response for procedures defined after it on the router. Middleware can also modify the context type which is passed to future procedures which is super powerful.

The middleware APIs are still fairly new. Better documentation will come in the future once they are more stable.

# Context switching

Middleware are allowed to modify the context. This includes being able to change it's type. All operations below the middleware in the router will receive the new context type.

```rust
use rspc::Router;

fn main() {
    let router = Router::<()>::new()
        .middleware(|mw| mw.middleware(|mw| async move {
            let old_ctx: () = mw.ctx;
            Ok(mw.with_ctx(42))
        }))
        .query("version", |t| {
            t(|ctx: i32, _: ()| "1.0.0")
        })
        .query("anotherQuery", |t| t(|ctx: i32, _: ()| "Hello World!"))
        .build();
}
```

# Route metadata

Feature coming soon. Tracking in issue [#21](https://github.com/oscartbeaumont/rspc/issues/21).

# Examples

## Logger middleware

```rust
let router = <Router>::new()
    // Logger middleware
    .middleware(|mw| {
        mw.middleware(|mw| async move {
            let state = (mw.req.clone(), mw.ctx.clone(), mw.input.clone());
            Ok(mw.with_state(state))
        })
        .resp(|state, result| async move {
            println!(
                "[LOG] req='{:?}' ctx='{:?}'  input='{:?}' result='{:?}'",
                state.0, state.1, state.2, result
            );
            Ok(result)
        })
    });
```

## Authentication middleware

```rust
pub struct UnauthenticatedContext {
    pub session_id: Option<String>,
}

let router = Router::<UnauthenticatedContext>::new()
    .query("unauthenticatedQuery", |t| {
        t(|ctx: UnauthenticatedContext, _: ()| {
            "Some Public Data!"
        })
    })
    .middleware(|mw| {
        mw.middleware(|mw| async move {
            match mw.ctx.session_id {
                Some(ref session_id) => {
                    let user = db_get_user_from_session(session_id).await;
                    // We use `.with_ctx` to switch the context type.
                    Ok(mw.with_ctx(AuthenticatedCtx { user }))
                }
                None => Err(rspc::Error::new(
                    ErrorCode::Unauthorized,
                    "Unauthorized".into(),
                )),
            }
        })
    })
    .query("authenticatedQuery", |t| {
        // This query takes the context from the middleware.
        t(|ctx: AuthenticatedCtx, _: ()| {
            "Some Secure Data!"
        })
    });
```

## Reject all middleware

```rust
let router = <Router>::new()
    // Reject all middleware
    .middleware(|mw| {
        mw.middleware(|mw| async move {
            Err(rspc::Error::new(
                ErrorCode::Unauthorized,
                "Unauthorized".into(),
            )) as Result<MiddlewareContext<_>, _>
        })
    })
    // The middleware was stop this query from being called.
    .query("unreachableQuery", |t| {
        t(|ctx: (), _: ()| {
            "Some Unreachable Data!"
        })
    });
```
