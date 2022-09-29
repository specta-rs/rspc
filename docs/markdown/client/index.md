---
title: Create Vanilla Client
header: Vanilla Client
index: 30
---

The vanilla client allows you to consume your API on the frontend. This client is the minimal core and it is recommended that you use the [React](/client/react) or [Solid](/client/solid) integration for building application.

To get started first install the minimal runtime package.

```bash
npm i @rspc/client
```

Next you need to export the Typescript bindings from your `rspc::Router` by using either [export_ts_bindings](/server/router#exporting-the-typescript-bindings) or [export_ts](/server/router#exporting-the-typescript-bindings).

```rust
let router = <rspc::Router>::new()
  // This will automatically export the bindings to the `./ts` directory when you run build() in a non-release Rust build
  .config(Config::new().export_ts_bindings("./bindings.rs"))
  .build();
```

Then you can use the `@rspc/client` package to consume your API.

```ts
import { createClient, FetchTransport } from "@rspc/client";
import type { Procedures } from "./ts/index"; // These were the bindings exported from your Rust code!

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Procedures>({
  // Refer to the integration your using for the correct transport.
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

// Now use the client in your code!
const version = await client.query(["version"]); // The types will be inferred from your backend.
const userOne = await client.query(["getUser", 1]);
const userTwo = await client.mutation(["addUser", { name: "Monty Beaumont" }]);
```

# Transports

rspc has multiple different transports which can be used.

```ts
import { createClient, FetchTransport, WebsocketTransport, NoOpTransport } from "@rspc/client";
import type { Procedures } from "./bindings.ts"; // The bindings exported from your Rust code!

const fetchClient = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const wsClient = createClient<Procedures>({
  transport: new WebsocketTransport("ws://localhost:8080/rspc/ws"),
});


const noOpClient = createClient<Procedures>({
  transport: new NoOpTransport(),
});
