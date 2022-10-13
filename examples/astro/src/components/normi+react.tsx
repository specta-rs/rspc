import { createClient, createWSClient, httpLink, wsLink } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { normiLink } from "@rspc/normi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";

import type { Procedures } from "../../../bindings";

export const rspc = createReactQueryHooks<Procedures>();

export const fetchQueryClient = new QueryClient();
const fetchClient = createClient<Procedures>({
  links: [
    normiLink({ queryClient: fetchQueryClient, contextSharing: true }),
    // TODO: Use batch link
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
    normiLink({ queryClient: fetchQueryClient, contextSharing: true }),
    wsLink({
      client: wsClient2,
    }),
  ],
});

function Example() {
  const { data: version } = rspc.useQuery(["version"]);
  const { data: user } = rspc.useQuery(["user"]);
  const { data: org } = rspc.useQuery(["org"]);

  console.log(user);
  console.log(org);

  console.log(window.normiCache);

  return (
    <div
      style={{
        border: "black 1px solid",
      }}
    >
      <h1>Version: {version}</h1>
    </div>
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
        <QueryClientProvider client={fetchQueryClient} contextSharing={true}>
          <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
            <Example />
          </rspc.Provider>
        </QueryClientProvider>
        {/* <rspc.Provider client={wsClient} queryClient={wsQueryClient}>
          <QueryClientProvider client={wsQueryClient}>
            <Example name="Websocket Transport" />
          </QueryClientProvider>
        </rspc.Provider> */}
      </div>
    </React.StrictMode>
  );
}
