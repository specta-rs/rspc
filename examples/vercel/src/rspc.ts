import { initRspc, httpLink } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react-query";
import { QueryClient } from "@tanstack/react-query";
import type { Procedures } from "./bindings";

const PROTOCOL = process.env.NODE_ENV === "development" ? "http" : "https";
const PATH = "api/rspc";
const HOST =
  typeof window === "undefined" ? process.env.VERCEL_URL : window.location.host;

export const client = initRspc<Procedures>({
  links: [
    httpLink({
      url: `${PROTOCOL}://${HOST}/${PATH}`,
      batch: true,
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
} = createReactQueryHooks<Procedures>(client);
