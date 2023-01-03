/** @jsxImportSource solid-js */
import { createWSClient, httpLink, wsLink } from "@rspc/client";
import { createSolidQueryHooks } from "@rspc/solid";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { createSignal } from "solid-js";

import type { Procedures } from "../../../bindings";

export const rspc = createSolidQueryHooks<Procedures>();

export const fetchQueryClient = new QueryClient();
const fetchClient = rspc.createClient({
  // onError(opts) {
  //   console.error("A", opts);
  // },
  links: [
    // loggerLink(),
    httpLink({
      url: "http://localhost:4000/rspc",
    }),
    // TODO: Support batching server-side
    // httpBatchLink({
    //   url: "http://localhost:4000/rspc",
    // }),
  ],
});

// TODO: Remove this abstraction or keep it?
const wsClient2 = createWSClient({
  url: "ws://localhost:4000/rspc/ws",
});

export const wsQueryClient = new QueryClient();
const wsClient = rspc.createClient({
  // onError(opts) {
  //   console.error("B", opts);
  // },
  links: [
    // loggerLink(),
    wsLink({
      client: wsClient2,
    }),
  ],
});

function Example({ name }: { name: string }) {
  const [rerenderProp, setRendererProp] = createSignal(Date.now().toString());
  const { data: version } = rspc.createQuery(() => ["version"]);
  const { data: transformMe } = rspc.createQuery(() => ["basic.transformMe"]);
  const { data: echo } = rspc.createQuery(() => [
    "basic.echo",
    "Hello From Frontend!",
  ]);
  const { mutate, isLoading } = rspc.createMutation("basic.sendMsg");
  const { error } = rspc.createQuery(() => ["basic.error"], {
    retry: false,
    onSuccess(v) {
      console.log("WHY", v);
    },
    onError(err) {
      console.error("A", err);
    },
  });

  return (
    <div
      style={{
        border: "black 1px solid",
      }}
    >
      <h1>{name}</h1>
      <p>Using rspc version: {version}</p>
      <p>Echo response: {echo}</p>
      <p>
        Error returned: {error?.code} {error?.message}
      </p>
      <p>Transformed Query: {transformMe}</p>
      <ExampleSubscription rerenderProp={rerenderProp()} />
      <button onClick={() => setRendererProp(Date.now().toString())}>
        Rerender subscription
      </button>
      <button onClick={() => mutate("Hello!")} disabled={isLoading}>
        Send Msg!
      </button>
    </div>
  );
}

function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
  const [i, setI] = createSignal(0);
  rspc.createSubscription(() => ["subscriptions.pings"], {
    onData(msg) {
      setI((i) => i + 1);
    },
  });

  return (
    <p>
      Pings received: {i} {rerenderProp}
    </p>
  );
}

export default function App() {
  return (
    <div style="background-color: rgba(255, 105, 97, .5);">
      <h1>React</h1>
      <QueryClientProvider client={fetchQueryClient} contextSharing={true}>
        <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
          <Example name="Fetch Transport" />
        </rspc.Provider>
      </QueryClientProvider>
      <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
        <QueryClientProvider client={wsQueryClient}>
          <Example name="Websocket Transport" />
        </QueryClientProvider>
      </rspc.Provider>
    </div>
  );
}
