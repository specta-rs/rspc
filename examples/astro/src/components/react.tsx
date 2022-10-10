import {
  createClient,
  createWSClient,
  httpLink,
  loggerLink,
  Observable,
  observableToPromise,
  wsLink,
} from "@rspc/client";
// import { createReactQueryHooks } from "@rspc/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React, { useEffect, useState } from "react";

// import type { Procedures } from "../../../bindings";
type Procedures = any; // TODO

// export const rspc = createReactQueryHooks<Procedures>();

export const fetchQueryClient = new QueryClient();
const fetchClient = createClient<Procedures>({
  links: [
    loggerLink(),
    httpLink({
      url: "http://localhost:4000/rspc",
    }),
  ],
});

// TODO: Remove this abstraction or keep it?
const wsClient2 = createWSClient({
  url: "ws://localhost:4000/rspc/ws",
});

export const wsQueryClient = new QueryClient();
const wsClient = createClient<Procedures>({
  links: [
    loggerLink(),
    wsLink({
      client: wsClient2,
    }),
  ],
});

// function Example({ name }: { name: string }) {
//   const [rerenderProp, setRendererProp] = useState(Date.now().toString());
//   const { data: version } = rspc.useQuery(["version"]);
//   const { data: transformMe } = rspc.useQuery(["transformMe"]);
//   const { data: echo } = rspc.useQuery(["echo", "Hello From Frontend!"]);
//   const { mutate, isLoading } = rspc.useMutation("sendMsg");
//   const { error } = rspc.useQuery(["error"], {
//     retry: false,
//   });

//   return (
//     <div
//       style={{
//         border: "black 1px solid",
//       }}
//     >
//       <h1>{name}</h1>
//       <p>Using rspc version: {version}</p>
//       <p>Echo response: {echo}</p>
//       <p>
//         Error returned: {error?.code} {error?.message}
//       </p>
//       <p>Transformed Query: {transformMe}</p>
//       <ExampleSubscription rerenderProp={rerenderProp} />
//       <button onClick={() => setRendererProp(Date.now().toString())}>
//         Rerender subscription
//       </button>
//       <button onClick={() => mutate("Hello!")} disabled={isLoading}>
//         Send Msg!
//       </button>
//     </div>
//   );
// }

// function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
//   const [i, setI] = useState(0);
//   rspc.useSubscription(["pings"], {
//     onData(msg) {
//       setI((i) => i + 1);
//     },
//   });

//   return (
//     <p>
//       Pings received: {i} {rerenderProp}
//     </p>
//   );
// }

export default function App() {
  useEffect(() => {
    console.log("HERE");
    fetchClient.query("version").then(console.log);
    wsClient.query("version").then(console.log);
  }, []);

  return (
    <React.StrictMode>
      <div
        style={{
          backgroundColor: "rgba(50, 205, 50, .5)",
        }}
      >
        <h1>React</h1>
        {/* <QueryClientProvider client={fetchQueryClient} contextSharing={true}>
          <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
            <Example name="Fetch Transport" />
          </rspc.Provider>
        </QueryClientProvider>
        <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
          <QueryClientProvider client={wsQueryClient}>
            <Example name="Websocket Transport" />
          </QueryClientProvider>
        </rspc.Provider> */}
      </div>
    </React.StrictMode>
  );
}
