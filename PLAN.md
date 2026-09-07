# rspc v2 — plan to a usable, released library

**Status: plan. Agreed intent, not shipped code.** Where this disagrees with the code, the
code wins.

## Why this document exists

Upstream rspc is discontinued — see the notice in `README.md` and
[discussion #351](https://github.com/specta-rs/rspc/discussions/351). Upstream had started
a v2 rewrite and abandoned it half-built, with the new code parked in `next/` folders
beside the v1 code it was meant to replace.

This fork picked that up. Read in order, the branch tells a clear story:

| Commit                             | What it did                                                                                                                     |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `8f46a9e` "Sadge"                  | Added the upstream-is-dead notice. Everything after is ours.                                                                        |
| `cffeabb` "axum sse, no websocket" | **The pivot.** Built a second, simpler transport — plain HTTP + SSE — and a new v2 client for it, setting the JSON-RPC/WS path aside. |
| `07a5e60`…`39e6796`                | Batching, streaming, flushing, and the first real integration tests.                                                                |
| `9038d26`…`a2e4ced`                | A working SolidJS + TanStack Query binding on the v2 client.                                                                        |

What follows is what it takes to go from "works for its author" to "a library someone else
can pick up, read the docs, install from a registry, and use without reading the source."

That gap is the subject of this document. It is not mostly a coding problem — it is roughly
a third correctness, a third completeness across transports and clients, and a third the
things libraries need that private forks skip: backward compatibility, docs, tests, CI,
releases, and a name.

### The governing constraint: we inherit upstream's users

rspc has existing users on 0.3.x. Upstream is gone, so **this fork is the only place they
can migrate to**. That makes backward compatibility a feature of the product, not
housekeeping:

- `crates/legacy` is *"the rspc 0.3.1 syntax implemented on top of the 0.4.0 core… allows
  incremental migration from the old syntax to the new syntax with minimal breaking
  changes"* (`crates/legacy/src/lib.rs:1-3`).
- `rspc/src/legacy.rs:21` implements `From<rspc_legacy::Router<TCtx>> for crate::Router<TCtx>`
  — a v1 router mounts straight into a v2 one, so a large app can migrate procedure by
  procedure rather than in one commit.
- Upstream deliberately made it a default feature (`72336dc "bring back legacy feature but
  make it default"`).

**The in-flight working-tree change deleting `crates/legacy` should not land.** It is the
right call for a private consumer that never used v1 syntax; it is the wrong call for the
library, because it deletes the only bridge our inherited users have. See Phase A.

The same logic applies to the v1 client and the JSON-RPC/WebSocket transport: they are not
dead weight to be cleared away, they are the compatibility surface.

---

## 1. Ground truth: where things actually stand

### 1.1 Feature matrix

This is the spine of "all use cases covered." Everything below is what the code does today,
not what it intends.

**Server — core**

| Capability                            | State | Evidence                                                                                    |
| ------------------------------------- | ----- | --------------------------------------------------------------------------------------------- |
| `query` / `mutation`                  | ✅    | `rspc/src/procedure/builder.rs:68-98`                                                        |
| `subscription`                        | ✅    | `builder.rs:113-129`                                                                         |
| `rspc::Stream` inside a query         | ✅    | Collects to a list client-side, documented at `rspc/src/stream.rs:16`                        |
| Router `procedure` / `nest` / `merge` | ✅    | `rspc/src/router.rs`; duplicate-key detection is the one thing with a test                   |
| v1 router → v2 router bridge          | ✅    | `rspc/src/legacy.rs:21`. **The migration path.** Behind the opt-in `legacy` feature          |
| Middleware on query/mutation          | 🟡    | Structurally complete, **zero tests or examples** exercise `.with()` end-to-end              |
| Middleware on subscriptions           | ❌    | `MiddlewareHandler` is Future-only (`middleware/middleware.rs:31-40`) — it can wrap the future that *produces* a stream, but cannot see, transform or short-circuit yielded items |
| Middleware context switching          | 🟡    | Wired through the generics, never tested                                                     |
| Error mapping across layers           | ❌    | One `TError` is fixed for the whole chain; a middleware cannot convert error types            |
| Procedure metadata (`meta.name()`)    | ❌    | Always returns the literal string `"todo"` (`rspc/src/procedure.rs:80,93`)                    |
| Backpressure / manual flush           | ❌    | `flush()` is a public no-op: `CAN_FLUSH` is never set true, `SHOULD_FLUSH` never read (`crates/procedure/src/stream.rs:17-37`) |
| Typed error → wire                    | 🟡    | Works, but asymmetric — user errors serialize bare while framework errors get a `{"~rspc":…}` envelope, so clients can't reliably discriminate (`crates/procedure/src/error.rs:99-114`) |

**Server — type export**

| Target                 | State | Evidence                                                                                           |
| ---------------------- | ----- | ---------------------------------------------------------------------------------------------------- |
| TypeScript (v2)        | 🟡    | Works, but subscriptions export the wrong type (§1.2b) and every `DataType`→string conversion is an `.unwrap()` (`languages/typescript.rs:203,211,219`) |
| TypeScript (v1 shape)  | ✅    | `ProceduresLegacy` emission, `#[cfg(feature = "legacy")]` at `typescript.rs:67-82`. Part of the compat surface |
| TypeScript source maps | 🟡    | Behind a flag that prints "unstable feature" at runtime                                              |
| Rust                   | ❌    | `languages/rust.rs` is `//! TODO: Bring this back when published.` plus ~85 commented-out lines. Enabling the `rust` feature compiles an empty module |
| OpenAPI                | ❌    | `crates/openapi` serves a static Swagger page; all route generation is commented out (`lib.rs:101-232`) |

**Transports** — richer than it first looks; the issue is wiring, not absence.

| Transport                | State | Evidence                                                                                             |
| ------------------------ | ----- | -------------------------------------------------------------------------------------------------------- |
| HTTP single (v2)         | ✅    | `integrations/axum/src/next.rs`                                                                          |
| HTTP batch (v2)          | 🟡    | Implemented, **untested** — `batch_query` is a commented-out stub at the end of `next.rs`                  |
| HTTP batch + streaming   | 🟡    | Custom `\d+:[…]\n` line protocol, undocumented and untested                                               |
| SSE subscriptions (v2)   | 🟡    | Works; no teardown, no reconnect policy (§1.4)                                                            |
| **WebSocket**            | 🟡    | **Implemented and v2-shaped, but not compiled in.** `endpoint.rs:41-61,204-309` has the upgrade + `handle_websocket`, and imports `rspc_procedure::{Procedure, Procedures}` — the v2 runtime types. Blocked only by `// mod endpoint;` at `lib.rs:9`, plus axum-0.7 path syntax (`"/:id"`, `endpoint.rs:33`) vs 0.8's `"/{id}"` |
| JSON-RPC wire format     | 🟡    | `jsonrpc.rs` + `jsonrpc_exec.rs`, compiled but currently unreachable. The wire format v1 clients speak    |
| Tauri IPC                | ✅    | `integrations/tauri` — the most complete integration in the repo, including abort support (`lib.rs:131-135`) |

**Clients**

| Package                | Targets | State | Notes                                                                                  |
| ---------------------- | ------- | ----- | ---------------------------------------------------------------------------------------- |
| `@rspc/client` (root)  | v1      | ✅    | `Transport` class with `FetchTransport` + `WebsocketTransport` (`transport.ts:76-132`), incl. reconnect. **Compat surface — keep** |
| `@rspc/client/next`    | v2      | 🟡    | The v2 client. No unsubscribe, no abort, no WS executor (§1.4)                            |
| `@rspc/solid-query`    | v2      | ✅    | The only framework binding ported — and the reference architecture (§1.8)                 |
| `@rspc/react-query`    | v1      | 🟡    | Works on v1; needs a v2 port alongside                                                    |
| `@rspc/svelte-query`   | v1      | ❌    | Needs a v2 port, **and** `peerDependencies.svelte` is `">=3 <5"` — Svelte 5 users cannot install it at all |
| `@rspc/query-core`     | v1      | 🟡    | Shared helpers for the **v1** bindings                                                    |
| `@rspc/tanstack-query` | —       | ❌    | Currently a byte-identical stale copy of `query-core`, imported by nothing. **The name is right and the slot is needed** — this should become the shared **v2** layer (§1.8) |
| `@rspc/tauri`          | both    | ✅    | v1 entrypoint wraps the v2 executor — the one place they're bridged cleanly               |
| `rspc-client` (Rust)   | —       | ❌    | Expects the JSON-RPC envelope `{"result":{"type":…,"data":…}}`; `next.rs` returns the bare value. Wire-incompatible with the v2 HTTP transport (though it would work against the JSON-RPC one) |

**Satellite crates**

| Crate          | State | Notes                                                                                                    |
| -------------- | ----- | ---------------------------------------------------------------------------------------------------------- |
| `validator`    | ✅    | Small, self-contained, exercised by `examples/core`                                                       |
| `tracing`      | 🟡    | `todo!()` for stream results (`traceable.rs:24`); imported by `examples/core` but never invoked           |
| `invalidation` | ❌    | Only `Invalidate::One` works; `Any`/`Many` are `todo!()`. Target name **hardcoded to `"sfmPost"`** with the comment *"Don't do this once `meta.name()` is correct"* |
| `cache`        | ❌    | Cache key is the literal `"todo"` (`lib.rs:47`) — every cached procedure shares one slot. `ttl` ignored (`memory.rs:18-21`) |
| `zer` (auth)   | ❌    | `.unwrap()`s on malformed/expired JWTs instead of returning `UnauthorizedError`; disables required claim validation with the comment *"This is very insecure!"* |
| `binario`      | ❌    | Blocked upstream: the `binario` crate returns non-`Send` futures. Currently stubbed to `todo!()`          |
| `devtools`     | ❌    | `mount()` is `todo!()`; implementation commented out                                                       |
| `openapi`      | ❌    | See above                                                                                                  |
| `client`       | ❌    | See above                                                                                                  |
| `legacy`       | ✅    | **The v1 compat layer.** Being deleted in the working tree — see §1.3                                     |

### 1.2 Correctness bugs

**(a) Malformed input double-panics the server.** The chain, fully traced:

1. `rspc/src/procedure.rs:101` — `TInput::from_input(input).unwrap()  // TODO: Error handling`
2. `from_input` correctly returns `Err(ProcedureError::Deserialize)` (`resolver_input.rs:52`) — so the `.unwrap()` panics.
3. `crates/procedure/src/procedure.rs:47-50` — `catch_unwind` turns it into `ProcedureError::Unwind`.
4. `From<ProcedureError> for ProcedureStream` (`stream.rs:60-68`) → `Inner::Value(Some(err))`, `flush: None`.
5. The integration polls it → `stream.rs:382-389`:
   ```rust
   Inner::Value(v) => {
       if self.flush.is_none() {
           // Poll::Ready(v.take().map(Err))
           todo!();
   ```
   **Second panic, uncaught.**

The correct implementation is the commented-out line directly above it. And `next.rs:490-493`
already has a `ProcedureError::Deserialize → 400` arm that **has never once executed**,
because the `.unwrap()` fires first. Any client sending a bad payload to any procedure kills
the request task.

**(b) Subscriptions export the wrong type.** `resolver_output.rs:88-90` returns
`<Vec<T>>::data_type(types)`. `ResolverOutput` is implemented once for `crate::Stream<S>`
and reused for both `rspc::Stream`-in-a-query (where `Vec` is correct and documented) and
real subscriptions (where it is wrong). A subscription yielding `T` exports as `T[]`; when
`T` is itself a list you get `string[][]`. Visible in this repo's own `examples/bindings.ts`:
`basicSubscription: { kind: "subscription", …, output: number[] }`.

The fix exists on the abandoned `origin/collect_procedure_types` branch —
`- <Vec<T>>::data_type(types)` / `+ T::data_type(types)`. That branch is stale (based on
`ab79d6e`, months behind) so it can't merge wholesale, but the change is directly portable.
`ProcedureKind` is in scope both where `ProcedureType` is built (`procedure.rs:111-116`) and
at export (`typescript.rs:191`).

**(c) Further reachable `todo!()`s.** `stream.rs:463` (mid-stream serialization failure);
`Debug` impls that are pure `todo!()` at `stream.rs:477`, `dyn_input.rs:75`,
`dyn_output.rs:62` — so any tracing or devtools layer that `{:?}`-prints them panics.

**(d) `State::get_mut` returns `&T`, not `&mut T`** (`crates/procedure/src/state.rs:54-58`)
— it calls `downcast_ref`. A copy-paste bug; the method is unusable as named.

**(e) `ProcedureError::NotFound` is never constructed anywhere**, and both integrations
handle it with `unimplemented!()` (`next.rs:496`). Unknown-procedure is handled out-of-band
by the HTTP layer. Either wire it up or delete the variant.

### 1.3 The workspace: how it broke, and what was done

An in-flight deletion of `crates/legacy/**` was half applied — the crate was gone but
`examples/legacy` still declared `rspc-legacy = { path = "../../crates/legacy" }` and
`features = ["legacy"]`. Cargo resolves the whole workspace manifest graph up front, so
**every `cargo` command failed**, including on unrelated crates.

Resolved by reverting the deletion, per §"The governing constraint" — `crates/legacy` is the
v1 migration bridge, not dead weight. Specifically:

- `crates/legacy/**` restored; `rspc-legacy` is an optional dependency again.
- The `legacy` feature is back, but **opt-in rather than default**. Enabling it emits the
  extra `ProceduresLegacy` type into generated bindings, which consumers who never used the
  0.3 syntax don't want; users migrating from 0.3 turn it on explicitly. This is the
  `D-1`-adjacent call flagged in Phase A.2, and it is a behaviour change from upstream's
  `default = ["legacy"]` — safe to make now, before republishing under a new name.
- Two members are **parked via `exclude`**, because they genuinely cannot compile yet:
  `examples/legacy` (calls `rspc_axum::endpoint`, commented out of the axum crate — returns
  in Phase E) and `crates/binario` + `examples/binario` (the `binario` crate returns
  non-`Send` futures — Phase G). Both are excluded with a written reason rather than deleted.

The lesson worth keeping: this repo has enough feature-gated and commented-out code that a
change can look complete and still break the graph. CI building **across the feature matrix**
(Phase A.4) is what prevents a recurrence, not care.

### 1.4 Client-side gaps (v2 client)

**Subscriptions can never be torn down.** `client.ts:27-30` declares
`subscribe: (…) => Unsubscribable`, but `UntypedClient.subscription()` has no `return`
statement, and neither does `observable.ts`'s `subscribe()` — the callback signature
`(observer) => void` never captures a teardown function. `grep -rn "unsubscribe"
packages/client/src/next` finds only the two type declarations. Calling
`.subscribe(x).unsubscribe()` throws. Downstream, `solid-query/src/useSubscription.ts:22-60`
subscribes inside a `createEffect` with no `onCleanup` — because there is nothing to call.

Note the v1 client does **not** have this problem, which makes it a useful reference.

**SSE never closes on error.** `sseExecute.ts:51-53` calls `o.error(e)` but not
`sse.close()`. The observable marks itself done and drops later events, while the native
`EventSource` keeps auto-reconnecting to a stream nobody reads — holding a slot against the
browser's ~6-connections-per-origin cap.

**The batch loader is global.** `fetchExecute.ts:9-12` — `batchLoaders` is module-scope state
keyed only by `"query"`/`"mutation"`, not by URL or client instance. Two clients pointing at
different backends in the same tick: the second one's requests go to the first one's URL.
And a throw inside the `setTimeout` callback (including the literal `throw new Error("invalid
stream content!")` at `:123`) is an unhandled rejection in a timer — **every queued caller
hangs forever**.

**No cancellation, and no WS executor.** No `AbortController` is passed to any `fetch()`, and
`next/index.ts` exports only `fetchExecute` and `sseExecute` — so a v2 app cannot use the
WebSocket transport even once the server side is re-enabled.

### 1.5 Documentation

There is effectively none for v2.

- `README.md` still shows the **v1** API (`.query("version", |t| t(|ctx, input| …))`).
  Combined with everything else, it is the most misleading file in the repo — though note
  that with `crates/legacy` kept, that syntax does still work, which makes precision about
  "v1 syntax, supported via the legacy feature" more important, not less.
- The docs site is a **separate repo** and still describes v1. It is not built from this
  repo, so nothing here keeps it honest — see Phase H for the split between crate-level docs
  that ship with the code and content owed to that repo.
- Rustdoc is a field of stubs: ~100 `TODO` markers, most literally `/// TODO` in place of a
  doc comment. `middleware.rs:51-81` has 8 in a single doc block admitting the generics are
  undocumented. Every `ProcedureStream` constructor is `/// TODO`.
- `procedure.rs:12-14`: `TODO: Request flow overview` / `TODO: Explain, what a procedure is`.
- No migration guide — the single document every inherited user needs.

### 1.6 Tests, CI, release

- **Rust: 9 real tests.** 8 in `integrations/axum/src/next.rs` (single-procedure + SSE), 1
  router duplicate-key test. All 3 in `rspc/tests/typescript.rs` are commented out.
- **TypeScript: zero.** No `vitest.config.*` anywhere. `@rspc/client` has no `test` script
  and lists `vitest` as a runtime `dependency`. `next/index.test.ts` has **no assertions**.
- **CI: none.** No `.github/workflows` directory at all. `dependabot.yml` has `# TODO: Rust`
  and `# TODO: npm` as unfilled entries.
- **`publish.sh` is broken** — it `cd`s into `crates/legacy/` (fine again if the deletion is
  reverted) and publishes **zero** TypeScript packages.
- **Versions are a mishmash**: `rspc` 0.4.1, `rspc-axum` 0.3.0, `tauri-plugin-rspc` 0.2.2,
  `rspc-procedure` 0.0.1, eight satellites at 0.0.0 with `publish = false`; all TS packages
  at 0.3.1.
- **`specta` is pinned to `=2.0.0-rc.22`** — an exact pre-1.0 release candidate that the
  entire type-generation story rests on. A genuine blocker for claiming stability.

### 1.7 Genuinely dead code

Narrower than it looks, once WS and legacy are kept:

- `integrations/axum/src/request.rs` (67 lines) — orphaned, and its `deserialize()` ignores
  the request body entirely.
- `rspc/src/mod.rs` — orphan module file referencing a non-existent `infallible` module.
- `rspc/src/languages/rust.rs` — entirely commented out.
- Committed `dist/` directories, already out of sync with both `HEAD` and the working tree.

`packages/tanstack-query`'s *contents* are dead, but the package is not — it is the slot the
shared v2 layer should occupy (§1.8).

`endpoint.rs`, `jsonrpc.rs` and `jsonrpc_exec.rs` are **not** in this list — they are the
WebSocket/JSON-RPC path, to be re-enabled (Phase E), not removed.

### 1.8 The client packages need a shared v2 layer, and it doesn't exist yet

`@rspc/solid-query` is the only v2 binding, and it is **self-contained** — it does not use
`@rspc/query-core` at all. That was the right call while it was the only one, and the wrong
shape to copy three more times. Measuring what is actually framework-bound:

| File                       | Lines | Framework-specific?                                                                 |
| -------------------------- | ----- | ------------------------------------------------------------------------------------- |
| `createOptionsProxy.ts`    | 181   | **Barely.** Its only Solid dependency is `import * as tanstack from "@tanstack/solid-query"` for `skipToken` and the option types. `skipToken` and the option shapes also exist in `@tanstack/query-core` |
| `useSubscription.ts`       | 63    | **Genuinely.** `createEffect` + `createStore` from `solid-js`. There is no framework-agnostic way to write this |
| `index.ts`                 | 11    | No — re-exports                                                                       |

So roughly **180 of 255 lines are shareable** and are currently trapped inside the Solid
package. Porting React and Svelte by copying it means maintaining the options proxy, the
procedure-path proxy, and `inferInput`/`inferOutput`/`inferError` three times over — and
they will drift, exactly as `query-core` and `tanstack-query` already drifted into
byte-identical-but-separate copies.

The split the code is already pointing at:

```
@rspc/tanstack-query   ← shared, framework-agnostic. Built on @tanstack/query-core.
  createRSPCOptionsProxy()  → queryOptions / mutationOptions / subscriptionOptions objects
  inferInput / inferOutput / inferError

@rspc/{react,solid,svelte}-query   ← thin. Only what needs the framework's reactivity:
  useSubscription()   React: useEffect + useSyncExternalStore
                      Solid: createEffect + createStore   (exists today)
                      Svelte 5: $effect + runes
  provider / context, if the framework wants one
```

This works because TanStack Query v5's `queryOptions()` objects are framework-agnostic by
design — every framework's `useQuery`/`createQuery` consumes the same option object. So
"framework-specific packages" remain necessary, but they shrink to the subscription hook and
the context plumbing, which is the part that genuinely cannot be shared.

**Consequence for the plan: `@rspc/tanstack-query` should be repurposed, not deleted.** The
package name is right and the slot is needed; only its current contents (a stale copy of the
v1 `query-core`) are wrong.

---

## 2. The blocking decision: what is this library called?

**Nothing about releasing can be planned until this is settled, so settle it first.**

- On crates.io, `rspc`, `rspc-axum`, `rspc-procedure` are **upstream's**. We cannot publish
  to them.
- On npm, the `@rspc` scope is **upstream's**. We cannot publish `@rspc/client`.

Three honest options:

| Option                                  | Consequence                                                                                                                                            |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A. Never publish.** Keep the names.   | Consumers use git/path deps forever. Cheapest — but "other people can use it" is then false in practice: no `cargo add`, no `npm i`, no docs.rs, no semver. |
| **B. Rename and publish.**              | The only path to a genuinely usable library. Costs a rename pass and a one-time migration note. **Recommended.**                                          |
| **C. Ask upstream to transfer the names.** | Upstream is unmaintained, so realistically a long shot — but it costs one issue to ask before doing B, and inheriting the names would make migration far easier for existing users. **Worth trying first.** |

Two consequences to plan for now:

- **Version reset.** Publishing under a new name is a fresh `0.1.0`. Use the opportunity to
  put every crate and package on one synchronised version.
- **A migration guide is mandatory.** Anyone arriving has a v1 codebase and needs to know the
  new names, that `legacy` gives them the old syntax on the new core, which transports exist,
  and what the router API looks like now.

### Branch naming

`better-axum` describes the branch's *first* commit, not what it became — and now that
WebSockets are staying, it is doubly misleading. Rename to `v2` (not `clients-v2`: the client
rewrite is the most visible part but the branch also changes the core, the transport layer
and the integrations).

```bash
git branch -m better-axum v2
git push origin -u v2
# delete the old remote branch only after every consumer's dependency pin is updated
```

Once the release phase is green, **fast-forward `main` to it**. Upstream is dead; there's no
reason to keep finished work on a side branch. Do it last — a broken `main` is worse than a
stale one.

### Target layout: v2 at the root, v1 demoted to `legacy/`

`next/` is named for a migration whose "before" side we are **keeping**, which makes it an
actively confusing name — it reads as "unreleased" when it means "current". v2 is the future
main API, so it should live where the main API lives, and v1 should move out of its way
rather than out of the repo.

**TypeScript** — `packages/client/`:

```
src/
  index.ts          v2  (was src/next/index.ts)        →  "@rspc/client"
  client.ts         v2  (was src/next/client.ts)
  observable.ts     v2
  UntypedClient.ts  v2
  types.ts          v2
  executors/
    fetch.ts        v2  (was src/next/fetchExecute.ts)
    sse.ts          v2  (was src/next/sseExecute.ts)
    ws.ts           v2  (new — Phase E.3)
  legacy/
    index.ts        v1  (was src/client.ts + transport.ts + typescript.ts)
                                                       →  "@rspc/client/legacy"
```

So the export map inverts:

| Subpath                 | Before          | After           |
| ----------------------- | --------------- | --------------- |
| `@rspc/client`          | v1              | **v2**          |
| `@rspc/client/next`     | v2              | deprecated alias for `.`, one release |
| `@rspc/client/legacy`   | —               | **v1**          |

**Rust** — same move: `integrations/axum/src/next.rs` becomes the crate's main endpoint
module, and the JSON-RPC/WebSocket path (`endpoint.rs`, `jsonrpc.rs`, `jsonrpc_exec.rs`)
moves under a `legacy/` module rather than sitting at the top level with its successor.
`rspc/src/legacy.rs` and `crates/legacy` keep their names — they are already honestly named.

**This inverts the default, which is a breaking change for v1 users** — `@rspc/client` stops
meaning v1. That is exactly why it should happen at the rename-and-republish moment (§2):
under a new package name at a fresh `0.1.0`, nothing changes under anyone silently, and the
migration guide can state the mapping in one table. Doing it later, after people have
depended on the new package, would cost a major version for no benefit.

Keep `./next` as a deprecated alias for one release so existing imports don't break in the
same commit that moves the files. Aion, the current main consumer, imports
`@rspc/client/next` and becomes a one-line change.

**Sequencing:** do the move in Phase G, after the v2 client is actually finished (Phase F) —
moving files while their contents are still changing makes every diff unreadable. But decide
it now, because Phase H's docs must be written against the final layout, not the current one.

---

## 3. The plan

Ten phases. Each leaves the tree in a working state and has an **exit criterion** — something
checkable, not a feeling. Sizes are S/M/L.

### Phase A — Decide on compat, unbreak the build, add CI (S)

Nothing below is verifiable until `cargo` resolves, so this goes first, and CI lands with it
so nothing silently rots again.

1. ~~**Decide `D-1`: does v1 compat stay?**~~ **Done — yes.** See §"The governing constraint".
2. ~~**Revert the `crates/legacy` deletion**, restore the `legacy` feature, park what cannot
   compile.~~ **Done** — see §1.3 for exactly what was restored, what became opt-in, and what
   was excluded.
3. Delete the genuinely dead files from §1.7 (`request.rs`, `rspc/src/mod.rs`) — neither is
   compat surface. Leave `packages/tanstack-query` in place; it gets repurposed in Phase F.
4. Add `.github/workflows/ci.yml`: `cargo check --workspace`, `cargo test --workspace`,
   `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `pnpm typecheck`. Build
   with **and without** the `legacy` and `ws` features — feature-gated code that nobody
   compiles is how this repo got here.
5. Fill in `dependabot.yml`'s unconfigured Rust and npm ecosystems.
6. Add a CI job (or a `cargo check` line) for each **parked** member, so `examples/legacy`
   and `binario` fail loudly when their blocker is lifted rather than silently rotting.

**Exit:** CI green on a clean checkout, across the feature matrix.

### Phase B — Correctness (M)

The bugs that make the library unsafe in front of untrusted input.

1. **`procedure.rs:101`** — propagate instead of `.unwrap()`. `From<ProcedureError> for
   ProcedureStream` already exists, so this is a `match` with an early `return e.into()`.
2. **`stream.rs:385`** — implement the `Inner::Value` arm (`Poll::Ready(v.take().map(Err))`).
   This makes the `unreachable!()`s at `:401` and `:412` genuinely reachable; handle them in
   the same change rather than leaving a latent trap.
3. **Test the 400 path.** With 1 and 2, `next.rs:490-493` becomes reachable for the first
   time. Highest-value new test in the plan.
4. **`stream.rs:463`** — return a real error on map failure; resolves the open question at
   `:459-460` about surfacing serialization errors.
5. **Implement the three `Debug` impls** — `finish_non_exhaustive()` is fine; anything but a
   panic.
6. **Fix `State::get_mut`** to actually return `&mut T`.
7. **Resolve `ProcedureError::NotFound`** — construct it in the integrations, or delete the
   variant and its `unimplemented!()` arms.
8. **Settle the error envelope.** Decide whether user errors carry a discriminator so clients
   can distinguish a typed application error from a framework error, and document the wire
   shape. An API decision, not a cleanup — and it must stay compatible with what v1 clients
   expect over JSON-RPC.

**Exit:** no `todo!()`/`unimplemented!()` reachable from a request; clippy passes with the
`todo`/`panic` lints on.

### Phase C — Complete the server surface (L)

1. **Real procedure metadata.** Thread the key from `router.rs:146` into `ProcedureMeta`,
   deleting both `"todo"` literals. This unblocks `cache` and `invalidation`, which are
   broken *because* of it.
2. **Middleware on subscriptions.** Today it can only wrap the future that produces a stream.
   Decide and implement: a per-item hook, or an explicit documented statement that middleware
   sees stream creation only. Silently doing the latter while looking like the former is the
   worst option.
3. **Error mapping across middleware layers** — allow a layer to change `TError`, or document
   that the chain is monomorphic in its error type.
4. **Test middleware at all.** Context switching and stacking have zero coverage; these
   generics are intricate enough that "it compiles" is not evidence.
5. **Decide the fate of `flush()`.** A public API that does nothing. Either finish the
   backpressure mechanism — the `require_manual_stream()`/`flushable()` implementation exists
   on the abandoned `collect_procedure_types` branch and can be ported — or delete the public
   function and its dead `flush: Option<Waker>` plumbing.
6. **Router polish** — guard the `key.len() == 0` case flagged at `router.rs:150`, and make
   `DuplicateProcedureKeyError`'s fields accessible.

**Exit:** every public API either works or does not exist; no 🟡 left in "Server — core".

### Phase D — Complete the type-export story (M)

1. **Fix the subscription output type** (§1.2b) — kind-aware, so `subscription` exports `T`
   while `rspc::Stream`-in-a-query keeps exporting `Vec<T>`.
2. **Restore `rspc/tests/typescript.rs`** — uncomment, repair, add a subscription case, and
   cover the `ProceduresLegacy` output too so the compat bindings don't silently regress.
3. **Replace export-path `.unwrap()`s** (`typescript.rs:203,211,219`, plus the source-map
   writer) with real errors. Export usually runs in a build script; panicking there is a
   terrible failure mode.
4. **Decide on the Rust exporter.** Finish `languages/rust.rs` or delete the module and the
   feature — an advertised flag that does nothing is a bug report waiting to happen.
5. **Decide on OpenAPI.** Same call for `crates/openapi`.

**Exit:** bindings correct for all three procedure kinds in both v1 and v2 shapes, asserted
by test; no feature flag is a silent no-op.

### Phase E — Complete the transport layer (M)

The phase this plan originally got wrong. The WebSocket path is **present and v2-shaped**;
this is a port, not a rewrite.

1. **Re-enable `endpoint.rs`** — uncomment `mod endpoint;` at `lib.rs:9`, update axum-0.7
   path syntax (`"/:id"` → `"/{id}"`, `endpoint.rs:33`) and whatever else axum 0.8 moved,
   and replace the `.unwrap()` on the WS upgrade extractor (`endpoint.rs:47`).
2. **Decide the WS wire format** (`D-2`). `endpoint.rs` speaks JSON-RPC via `jsonrpc_exec`.
   Either keep that as the compatibility transport (v1 clients keep working unchanged), or
   also offer WS speaking the newer shape. Keeping JSON-RPC for WS and the new shape for
   HTTP+SSE is coherent, but must be documented rather than discovered.
3. **Add `wsExecute` to the v2 client** so v2 apps can use WebSockets. `transport.ts:76-132`'s
   `WebsocketTransport`, including its reconnect handling, is the reference implementation.
4. **Make the `ws` feature honest.** It currently compiles and does nothing because the module
   is commented out. After (1) it becomes real — and CI must build both with and without it.
5. **Document all four transports** side by side with their trade-offs: HTTP (no
   subscriptions), SSE (server→client only, ~6 connections per origin, no custom headers),
   WebSocket (bidirectional, reconnect complexity), Tauri IPC. Users currently have no way to
   choose.
6. **Test the batch executor** — `batch_query` is a commented-out stub, and batching is the
   most intricate untested code in the repo.

**Exit:** every transport in §1.1 is either ✅ or explicitly out of scope in the docs.

### Phase F — Complete the client story (L)

The largest phase, and where "all clients" lives.

1. **Fix the v2 core client** (`packages/client/src/next`):
   - Give `observable` a teardown contract — `(observer) => (() => void) | void`, with
     `subscribe()` returning `{ unsubscribe }`. Everything else depends on this.
   - `UntypedClient.subscription()` returns the handle, making the declared type honest.
   - `sseExecute` returns a teardown that closes the `EventSource`, closes it on `onerror`
     too, and has a documented reconnect policy.
   - `fetchExecute`: key batch loaders per client instance, not per module; wrap the timer
     body so failures reject every queued caller instead of hanging them; thread an
     `AbortController` through every request.
   - Implement or explicitly reject the `batch: false, stream: true` mode its own comment says
     is "not implemented yet".
   - Document the batch wire protocol (`\d+:[…]\n`), which currently exists only as a regex in
     one file and a formatter in another.
2. **Unify type inference helpers.** `inferInput`/`inferOutput`/`inferError` exist **only
   inside `@rspc/solid-query`** and operate on options-proxy objects, while v1's
   `inferQueryResult<TProcedures, K>` lives in `client/src/typescript.ts` with an incompatible
   two-argument signature. Promote one set into `@rspc/client` so every binding shares it, and
   keep the v1 names working.
3. **Extract the shared v2 layer first, then port the bindings** (§1.8). Doing this in the
   other order means writing the options proxy three more times and then merging them back.
   - **`@rspc/tanstack-query` becomes the shared, framework-agnostic v2 package.** Move
     `createRSPCOptionsProxy`, the procedure-path proxy and `inferInput`/`inferOutput`/
     `inferError` out of `solid-query/src/createOptionsProxy.ts` (~180 of its 255 lines), and
     re-point its one framework import from `@tanstack/solid-query` to `@tanstack/query-core`.
   - **`@rspc/solid-query` shrinks** to `useSubscription` plus re-exports — proving the split
     against the one binding already known to work, before porting anything.
   - **`@rspc/react-query`** — v2 path: a `useSubscription` on `useEffect` +
     `useSyncExternalStore`, over the shared proxy.
   - **`@rspc/svelte-query`** — v2 path: a `$effect`-based subscription hook, **and** bump
     `peerDependencies.svelte` from `">=3 <5"` and move off Svelte-4 `export let` to runes.
     Svelte 5 users currently cannot install this package at all.
   - **`@rspc/query-core`** stays as the **v1** shared layer, so the v1 bindings keep working.
   - Decide whether Vue/Angular bindings are in scope, or explicitly out (`D-6`). With the
     shared layer extracted, each is roughly a subscription hook.
4. **Decide the Rust client's fate** (`D-3`). It cannot talk to the v2 HTTP transport, though
   it would work against the JSON-RPC one. Rewrite against the v2 wire format, re-point it at
   the JSON-RPC transport, or delete it — but do not leave a broken client in the repo.
5. **Keep the v1 client documented and tested**, not merely present. It is the compat surface;
   if it breaks silently, migration breaks silently.

**Exit:** every framework has a v2 binding; every v1 binding still works and says so.

### Phase G — Clean up (S)

Deliberately small, because most of what looked like dead code is compat surface.

1. Delete the §1.7 list, if not already done in Phase A.
2. **Triage the satellite crates.** Fix, or remove from the default workspace and say so in
   the README. Do not ship crates that panic on their only entry point.
   - `validator` — keep.
   - `tracing`, `invalidation`, `cache` — fixable, and the latter two become fixable *because
     of* Phase C.1.
   - `zer` — fix the `.unwrap()`s and the disabled claim validation, or park it. An auth crate
     that panics on a malformed token must not ship as-is.
   - `binario` — blocked upstream on non-`Send` futures. Park with a written reason; do not
     land the `todo!()` stub currently in the working tree.
   - `devtools`, `openapi`, `client` — finish or delete.
3. **Move v2 to the root and demote v1 to `legacy/`**, per the target layout in §2. Do this
   as a pure file move plus export-map change in its own commit — no behaviour changes mixed
   in, so the diff stays reviewable. Land the deprecated `./next` alias in the same commit.
4. **Clean the build outputs** — gitignore `dist/` and build on publish, or regenerate in CI.

**Exit:** every file is reachable from a public entry point, a test, or an example, and the
layout matches what the docs describe.

### Phase H — Documentation (L)

The phase most likely to be skipped and least likely to be forgiven.

**The docs site lives in its own repo**, so this phase does not build one. It splits into
what belongs in *this* repo and what is content owed to the docs repo — and the two have
different failure modes: in-repo docs rot silently against the code, site content rots
silently against the release.

**In this repo:**

1. **Rewrite `README.md`** — it currently shows v1 syntax with no indication that it is v1.
   With `legacy` kept, that syntax still works, which makes being precise about *which* API
   is being shown more important, not less.
2. **Fill in the rustdoc.** ~100 `TODO` markers, most standing in for the doc comment itself.
   Priorities: the request-flow overview (`procedure.rs:12-14`), the `Middleware` generics
   (`middleware.rs:51-81` — 8 TODOs in one block), and every `ProcedureStream` constructor.
   This is the documentation that ships with the crate and renders on docs.rs, so it is the
   part that cannot be delegated to the site.
3. **Fix the examples.** They are the de-facto integration tests *and* the code the docs will
   link to: `examples/client` (wire-incompatible), `examples/binario` (panics),
   `examples/core`'s cache/invalidation/tracing/zer usage, and `examples/legacy` — which
   becomes a *supported* example again rather than something to delete. Add one example per
   transport and per client binding.

**Owed to the docs repo** (write it here as drafts if that's easier, but it ships there):

4. **Migration guide.** The highest-value document we can produce, and the one every
   inherited user needs: the new names, how to turn on `legacy` and mount a v1 router inside a
   v2 one (`rspc/src/legacy.rs:21`), how to migrate procedure by procedure, which transport to
   pick, and what actually breaks.
5. **Getting started** — install, define a router, mount on axum, export bindings, call from a
   client. One page, copy-pasteable, and kept in sync with a real example from (3).
6. **A compatibility matrix** — which client versions talk to which transports and wire
   formats. Today this is knowable only by reading `jsonrpc_exec.rs` and `next.rs` side by
   side.
7. **The wire formats** — JSON-RPC, v2 single, batch, batch+stream, SSE events, and the error
   envelope from Phase B.8. Anyone writing a non-JS client needs this; it exists nowhere.
8. **A guide per client package**, plus a recipe for "my framework has no binding" — which,
   after §1.8's split, is genuinely just a subscription hook over the shared proxy.

**Also:** point rspc.dev's successor at the right place, and make sure the archived upstream
docs are not the first search result people act on. A stale doc site that still describes v1
as current is worse than no site.

**Exit:** a stranger can install it, build a server and client, and migrate a v1 app without
reading the source. Every code sample in the docs repo corresponds to a compiling example in
this one.

### Phase I — Tests (M)

Runs alongside D–H, tracked separately so it doesn't get dropped.

1. Wire up TS testing at all — `vitest.config.ts`, `vitest` to `devDependencies`, `test`
   scripts, and real assertions in `next/index.test.ts` (it has none and would pass if every
   call silently failed).
2. Cover middleware end-to-end, including context switching (Phase C.4).
3. Cover the client: unsubscribe actually unsubscribes, SSE closes on error, batch failures
   reject rather than hang, abort works.
4. **Round-trip tests per transport** — a real server, a real client, every procedure kind,
   over HTTP, SSE and WebSocket. This is what would have caught the Rust client's wire
   incompatibility.
5. **A v1-compat regression test** — a v1-syntax router mounted via `legacy`, called by a v1
   client. Backward compatibility that isn't tested isn't backward compatibility.

**Exit:** every bug in §1.2 and §1.4 has a regression test.

### Phase J — Release (M)

1. **Settle the name** (§2) and apply it across crates, packages, docs and examples.
2. **Synchronise versions** — one version across all crates and packages, starting fresh.
3. **Address the `specta` RC pin.** `=2.0.0-rc.22` is an exact pre-1.0 pin under the whole
   type system. Wait for stable, relax the pin, or document that our stability is bounded by
   it.
4. **Rewrite `publish.sh`** — publish Rust crates in dependency order and TS packages via
   `pnpm publish -r`. It currently publishes zero TS packages.
5. **Automate release in CI**, with a dry-run on every PR.
6. **Add the crate metadata** every satellite is missing (`# TODO: Crate metadata & publish`):
   description, keywords, categories, repository, license, readme.
7. **Rename the branch to `v2`, then fast-forward `main`** (§2).
8. **Write a CHANGELOG** and state a support policy — what's stable, what's experimental,
   what MSRV, and **how long the v1 compat layer is supported**.

**Exit:** `cargo add <name>` and `npm i <scope>/client` work, and docs.rs renders.

---

## 4. Open decisions

**`D-1` — Does the v1 compatibility layer stay?**
**Recommendation: yes, and it is close to non-negotiable.** Upstream is gone, so this fork is
the only migration target its users have; `crates/legacy` plus `rspc/src/legacy.rs`'s
`From<rspc_legacy::Router>` is what lets them move incrementally instead of rewriting in one
commit. The cost is real — it keeps `jsonrpc*.rs`, the v1 client, `ProceduresLegacy` export
and `examples/legacy` alive, and every one needs tests and docs. But deleting it converts
"upstream died, here's where to go" into "upstream died, rewrite from scratch." Separately
decide whether it stays a *default* feature; opt-in under a new package name is defensible.

**`D-2` — What wire format do WebSockets speak?**
`endpoint.rs` speaks JSON-RPC. Options: keep WS as the JSON-RPC/compat transport only; port
WS to the v2 shape; or support both. Keeping JSON-RPC on WS is the cheapest and preserves v1
clients unchanged — but then the v2 client's `wsExecute` must speak JSON-RPC too, which is
worth being explicit about before it's built.

**`D-3` — Does the Rust client survive?**
It cannot talk to the v2 HTTP transport but would work against JSON-RPC. Re-point, rewrite,
or delete — leaving it broken helps nobody.

**`D-4` — Is `rspc::Stream`-in-a-query worth keeping?**
It is the direct cause of the subscription type bug, because one `ResolverOutput` impl serves
two different semantics. A genuinely nice feature, but worth asking whether the complexity
earns its place before building more on it.

**`D-5` — What is the stability promise?**
Given the specta RC pin, `0.x` with a clear "expect breaking changes" note is more honest
than implying stability the dependency graph cannot back.

**`D-6` — Which frameworks are supported?**
React, Solid, Svelte 5 and Tauri is a defensible line, with a documented recipe for building
your own on `UntypedClient`. Decide explicitly rather than by which packages happen to exist.

---

## 5. Suggested order

```
A  compat decision + build + CI  ─→  B  correctness  ─→  C  server surface  ─→  D  type export
                                            │                    │
                                            │                    └─→ unblocks cache + invalidation
                                            ├─→  E  transports (port WS)  ─┐
                                            └─→  F  clients  ──────────────┴─→  G  cleanup
I  tests ─── runs alongside D–H, not after
                                                          H  docs  ─→  J  release
```

**A and B are the ones worth doing immediately.** They are small, and until they land the
repo does not build and the server panics on malformed input — no work elsewhere matters
while that is true. A also forces the compat decision, which changes the shape of E, F, G and
H, so it genuinely cannot be deferred.

**C through F are the bulk of "usable by other people."** F is the single largest piece,
because three framework bindings still need v2 ports and one of them cannot be installed by
Svelte 5 users at all.

**H is the phase most likely to be skipped.** A correct library whose README describes an API
the reader can't find reads, to a newcomer, exactly like a broken one — and with v1 syntax
still supported behind a feature flag, being precise about *which* API is which matters more
here than it would in most projects. The docs site being a separate repo makes this easier to
defer and harder to notice, which is a reason to schedule it, not a reason to relax.
