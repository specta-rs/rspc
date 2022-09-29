---
title: React
index: 31
---

rspc can be used on the frontend with [React](https://reactjs.org) via the powerful [React Query](https://tanstack.com/query/v4) library which provides caching, refetching and a lot more.

To get started first install the required packages.

```bash
npm i @rspc/client # The core client
pnpm i @rspc/react # The React integration
```

Then you can do the following:

```ts
import { QueryClient } from '@tanstack/react-query';
import { FetchTransport, createClient } from '@rspc/client';
import { createReactQueryHooks } from '@rspc/react';

import type { Procedures } from "./ts/index"; // These were the bindings exported from your Rust code!

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Procedures>({
  // Refer to the integration your using for the correct transport.
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createReactQueryHooks<Procedures>();

function SomeComponent() {
    const { data, isLoading, error } = rspc.useQuery(['version']);
    const { mutate } = rspc.useMutation('updateVersion');

    return (
        <>
            <p>{data}</p>
            <button onClick={() => mutate.mutate("newVersion")}>Do thing</button>
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