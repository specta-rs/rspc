---
title: SolidJS
---

# SolidJS

rspc can be used on the frontend with [SolidJS](https://www.solidjs.com/) via [solid-query](https://github.com/ardeora/solid-query) which provides caching, refetching and a lot more. 

To get started first install the required packages.

```bash
npm i @rspc/client # The core client
pnpm i @rspc/solid # The SolidJS integration
```

Then you can do the following:

```ts
import { QueryClient } from '@tanstack/react-query';
import { FetchTransport, createClient } from '@rspc/client';
import { createSolidQueryHooks } from '@rspc/solid';

import type { Procedures } from "./ts/index"; // These were the bindings exported from your Rust code!

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Operations>({
  // Refer to the integration your using for the correct transport.
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createSolidQueryHooks<Operations>();

function SomeComponent() {
    const echo = rspc.createQuery(["echo", "somevalue"]);
    const sendMsg = rspc.createMutation("sendMsg");

    return (
        <>
            <p>{echo.data}</p>
            <button onClick={() => sendMsg.mutate("newVersion")}>Do thing</button>
        </>
    )
}

function App() {
    return (
        <rspc.Provider client={client} queryClient={queryClient}>
            <SomeComponent />
        </rspc.Provider>
    )
}
````