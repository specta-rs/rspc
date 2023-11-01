import { initRspc, httpLink, wsLink } from "@rspc/client";
import { tauriLink } from "@rspc/tauri";
import { createReactQueryHooks } from "@rspc/react-query";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React, { useState } from "react";

// Export from Rust. Run `cargo run -p example-axum` to start server and export it!
import type { Procedures } from "../../../../bindings";

const fetchQueryClient = new QueryClient();
const fetchClient = initRspc<Procedures>({
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
  // onError // TODO: Make sure this is still working
});

const wsQueryClient = new QueryClient();
const wsClient = initRspc<Procedures>({
  links: [
    // loggerLink(),

    wsLink({
      url: "ws://localhost:4000/rspc/ws",
    }),
  ],
});

const tauriQueryClient = new QueryClient();
const tauriClient = initRspc<Procedures>({
  links: [
    // loggerLink(),

    tauriLink(),
  ],
});

// TODO: Allowing one of these to be used for multiple clients! -> Issue is with key mapper thing
// TODO: Right now we are abusing it not working so plz don't do use one of these with multiple clients in your own apps.
export const rspc = createReactQueryHooks<Procedures>();
// export const rspc2 = createReactQueryHooks<Procedures>(wsClient);

function Example({ name }: { name: string }) {
  const [rerenderProp, setRendererProp] = useState(Date.now().toString());
  const version = rspc.useQuery(["version"]);
  const transformMe = rspc.useQuery(["transformMe"]);
  const echo = rspc.useQuery(["echo", "Hello From Frontend!"]);
  const sendMsg = rspc.useMutation("sendMsg");
  const errorQuery = rspc.useQuery(["error"], {
    retry: false,
  });

  const [subId, setSubId] = useState<number | null>(null);
  const [enabled, setEnabled] = useState(true);

  rspc.useSubscription(["testSubscriptionShutdown"], {
    enabled,
    onData(msg) {
      setSubId(msg);
    },
  });

  return (
    <div
      style={{
        border: "black 1px solid",
      }}
    >
      <h1>{name}</h1>
      <p>Using rspc version: {version.data}</p>
      <p>Echo response: {echo.data}</p>
      <p>Error returned: {JSON.stringify(errorQuery.error)} </p>
      <p>Transformed Query: {transformMe.data}</p>
      <ExampleSubscription key={rerenderProp} rerenderProp={rerenderProp} />
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
        disabled={subId === null}
      />
      {`${enabled ? "Enabled" : "Disabled"} ${subId}`}
    </div>
  );
}

function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
  const [i, setI] = useState(0);
  rspc.useSubscription(["pings"], {
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
    <React.StrictMode>
      <div
        style={{
          backgroundColor: "rgba(50, 205, 50, .5)",
        }}
      >
        <h1>React</h1>
        <QueryClientProvider client={fetchQueryClient}>
          <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
            <Example name="Fetch Transport" />
          </rspc.Provider>
        </QueryClientProvider>
        <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
          <QueryClientProvider client={wsQueryClient}>
            <Example name="Websocket Transport" />
            <Demo />
          </QueryClientProvider>
        </rspc.Provider>
        <rspc.Provider client={tauriClient} queryClient={tauriQueryClient}>
          <QueryClientProvider client={tauriQueryClient}>
            <Example name="Tauri Transport" />
          </QueryClientProvider>
        </rspc.Provider>
      </div>
    </React.StrictMode>
  );
}

function Demo() {
  const a = rspc.useSubscription(["batchingTest"], {
    onData(msg) {
      console.log("A", msg);
    },
  });

  const b = rspc.useSubscription(["batchingTest"], {
    onData(msg) {
      console.log("B", msg);
    },
  });

  return null;
}
