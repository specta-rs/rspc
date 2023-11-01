/** @jsxImportSource solid-js */

import { createRSPCClient, httpLink, wsLink } from "@rspc/client";
import { tauriLink } from "@rspc/tauri";
import { createRSPCSolidQuery } from "@rspc/solid-query";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { createSignal } from "solid-js";

// Export from Rust. Run `cargo run -p example-axum` to start server and export it!
import type { Procedures } from "../../../../bindings";

const fetchQueryClient = new QueryClient();
const fetchClient = createRSPCClient<Procedures>({
  links: [
    // loggerLink(),

    httpLink({
      url: "http://localhost:4000/rspc",

      // You can enable batching -> This is generally a good idea unless your doing HTTP caching
      // batch: true,

      // You can override the fetch function if required
      // fetch: (input, init) => fetch(input, { ...init, credentials: "include" }), // Include Cookies for cross-origin requests

      // Provide static custom headers
      // headers: {
      //   "x-demo": "abc",
      // },

      // Provide dynamic custom headers
      // headers: ({ op }) => ({
      //   "x-procedure-path": op.path,
      // }),
    }),
  ],
});

const wsQueryClient = new QueryClient();
const wsClient = createRSPCClient<Procedures>({
  links: [
    // loggerLink(),

    wsLink({
      url: "ws://localhost:4000/rspc/ws",
    }),
  ],
});

const tauriQueryClient = new QueryClient();
const tauriClient = createRSPCClient<Procedures>({
  links: [
    // loggerLink(),

    tauriLink(),
  ],
});

// TODO: Allowing one of these to be used for multiple clients! -> Issue is with key mapper thing
// TODO: Right now we are abusing it not working so plz don't do use one of these with multiple clients in your own apps.
export const rspc = createRSPCSolidQuery(fetchClient);

function Example({ name }: { name: string }) {
  const [rerenderProp, setRendererProp] = createSignal(Date.now().toString());
  const version = rspc.createQuery(() => ["version"]);
  const transformMe = rspc.createQuery(() => ["transformMe"]);
  const echo = rspc.createQuery(() => ["echo", "Hello From Frontend!"]);
  const sendMsg = rspc.createMutation("sendMsg");
  const error = rspc.createQuery(() => ["error"], {
    retry: false,
  });

  const [subId, setSubId] = createSignal<number | null>(null);
  const [enabled, setEnabled] = createSignal(true);

  rspc.createSubscription(
    () => ["testSubscriptionShutdown"],
    () => ({
      enabled: enabled(),
      onData(msg) {
        setSubId(msg);
      },
    })
  );

  return (
    <div
      style={{
        border: "black 1px solid",
      }}
    >
      <h1>{name}</h1>
      <p>Using rspc version: {version.data}</p>
      <p>Echo response: {echo.data}</p>
      <p>Error returned: {JSON.stringify(error.error)} </p>
      <p>Transformed Query: {transformMe.data}</p>
      <ExampleSubscription rerenderProp={rerenderProp()} />
      <button onClick={() => setRendererProp(Date.now().toString())}>
        Rerender subscription
      </button>
      <button
        onClick={() => sendMsg.mutate("Hello!")}
        disabled={sendMsg.isPending}
      >
        Send Msg!
      </button>
      <br />
      <input
        type="checkbox"
        onClick={(e) => setEnabled((e.currentTarget as any).checked)}
        value="false"
        disabled={subId() === null}
      />
      {`${enabled() ? "Enabled" : "Disabled"} ${subId()}`}
    </div>
  );
}

function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
  const [i, setI] = createSignal(0);
  rspc.createSubscription(
    () => ["pings"],
    () => ({
      onData(_) {
        setI((i) => i + 1);
      },
    })
  );

  return (
    <p>
      Pings received: {i()} {rerenderProp}
    </p>
  );
}

export default function App() {
  return (
    <div style="backgroundColor: 'rgba(50, 205, 50, .5)'">
      <h1>Solid</h1>
      {/* TODO: rspc.Provider implies fetchClient??? */}
      <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
        <QueryClientProvider client={fetchQueryClient}>
          <Example name="Fetch Transport" />
        </QueryClientProvider>
      </rspc.Provider>
      <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
        <QueryClientProvider client={wsQueryClient}>
          <Example name="Websocket Transport" />
        </QueryClientProvider>
      </rspc.Provider>
      <rspc.Provider client={tauriClient} queryClient={tauriQueryClient}>
        <QueryClientProvider client={tauriQueryClient}>
          <Example name="Tauri Transport" />
        </QueryClientProvider>
      </rspc.Provider>
    </div>
  );
}
