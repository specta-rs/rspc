# rspc cache

[![docs.rs](https://img.shields.io/crates/v/rspc-cache)](https://docs.rs/rspc-cache)

> [!CAUTION]
> This crate is still a work in progress. You can use it but we can't guarantee that it's API won't change.

Provides a simple way to cache the results of rspc queries with pluggable backends.

Features:
 - Simple to use
 - Pluggable backends (memory, redis, etc.)
 - Configurable cache TTL

## Example

```rust
// TODO: imports

fn todo() -> Router2<Ctx> {
    Router2::new()
        .setup(CacheState::builder(Memory::new()).mount())
        .procedure("my_query", {
            <BaseProcedure>::builder()
                .with(cache())
                .query(|_, _: ()| async {
                    // if input.some_arg {}
                    cache_ttl(10);

                    Ok(SystemTime::now())
                })
        })
}
```
