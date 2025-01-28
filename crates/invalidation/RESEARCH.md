## Research

Some thoughts about the design from a while ago. This can probally be removed once we are happy the solution meets all of the requirements.

## Questions

For my own future reference: https://discord.com/channels/@me/813276814801764382/1263123489477361828

### Pull vs Push based invalidation events

Pull based is where the middleware is applied to the query.
Push based is where the middleware is applied to the mutation.

I think we want a pull-based so resources can define their dependencies a-la React dependencies array.

### Stream or not?

I'm leaning stream-based because it pushes the type safety concern onto the end user.

```rust
<BaseProcedure>::builder()
    // "Pull"-based. Applied to queries. (I personally a "Pull"-based approach is better)
    .with(rspc_invalidation::invalidation(
        |input, result, operation| operation.key() == "store.set",
    ))
    .with(rspc_invalidation::invalidation(
        // TODO: how is `input().id` even gonna work lol
        |input, result, operation| {
            operation.key() == "notes.update" && operation.input().id == input.id
        },
    ))
    // "Push"-based. Applied to mutations.
    .with(rspc_invalidation::invalidation(
        |input, result, invalidate| invalidate("store.get", ()),
    ))
    .with(rspc_invalidation::invalidation(
        |input, result, operation| invalidate("notes.get", input.id),
    ))
    // "Pull"-based but with stream.
    .with(rspc_invalidation::invalidation(|input: TArgs| {
        stream! {
            // If practice subscribe to some central event bus for changes
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                yield Invalidate; // pub struct Invalidate;
            }
        }
    }))
    .query(...)
```

### Exposing result of procedure to invalidation closure

If we expose result to the invalidate callback either the `Stream` or the value must be `Clone` which is not great, although the constrain can be applied locally by the middleware.

If we expose the result and use a stream-based approach do we spawn a new invalidation closure for every result? I think this is something we will wanna leave the user in control of but no idea what that API would look like.

### How do we get `BuiltRouter` into `Procedure`?

It kinda has to come in via context or we need some magic system within rspc's core. Otherwise we basically have a recursive dependency.

### Frontend?

Will we expose a package or will it be on the user to hook it up?

## Other concerns

## User activity

Really we wanna only push invalidation events that are related to parts of the app the user currently has active. An official system would need to take this into account somehow. Maybe some integration with the frontend router and websocket state using the `TCtx`???

## Data or invalidation

If we can be pretty certain the frontend wants the new data we can safely push it straight to the frontend instead of just asking the frontend to refetch. This will be much faster but if your not tracking user-activity it will be way slower because of the potential volume of data.

Tracking user activity pretty much requires some level of router integration which might be nice to have an abstraction for but it's also hard.

## Authorization

**This is why rspc can't own the subscription!!!**

We should also have a way to take into account authorization and what invalidation events the user is able to see. For something like Spacedrive we never had this problem because we are a desktop app but any web app would require this.
