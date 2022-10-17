import { createClient, createWSClient, httpLink, wsLink } from "@rspc/client";
import { createReactHooks } from "@rspc/react";
import { QueryClient } from "@tanstack/react-query";
import type { Procedures } from "../../bindings";

export const rspc = createReactHooks<Procedures>();

const wsClient = createWSClient({
  url: "ws://localhost:4000/rspc/ws",
});

export const client = rspc.createClient({
  links: [
    typeof window === "undefined"
      ? // WebsocketTransport can not be used Server Side, so we provide FetchTransport instead.
        // If you do not plan on using Subscriptions you can use FetchTransport on Client Side as well.
        httpLink({ url: "http://localhost:4000/rspc" }) // TODO: Switch to `httpBatchLink` when supported
      : wsLink({
          client: wsClient,
        }),
  ],
});

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false, // If you want to retry when requests fail, remove this.
    },
  },
});

export const {
  useContext,
  useMutation,
  useQuery,
  useSubscription,
  Provider: RSPCProvider,
} = rspc;
