import { createClient, FetchTransport, WebsocketTransport } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { QueryClient } from "@tanstack/react-query";
// import type { Procedures } from "../../bindings";

const PATH = 'api/rspc';
const HOST = typeof window === "undefined" ? process.env.VERCEL_URL : window.location.host ;

export const client = createClient<any>({
  transport: typeof window === "undefined" ? new FetchTransport(`http://${HOST}/${PATH}`) : new WebsocketTransport(`ws://${HOST}/${PATH}/ws`)
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
} = createReactQueryHooks<any>();
