---
title: TanStack Query
layout: ../../layouts/MainLayout.astro
---

rspc integrations with the wonderful [TanStack Query](https://tanstack.com/query/v4) to provide caching, refetching and a lot more.

To use rspc with TanStack Query you must do the following:

```tsx
import { QueryClient } from '@tanstack/react-query';
import { FetchTransport, createClient, createReactQueryHooks } from '@rspc/client';

import type { Operations } from "./ts/index"; // These were the bindings exported from your Rust code!

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Operations>({
  // Refer to the integration your using for the correct transport.
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createReactQueryHooks<Operations>();

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