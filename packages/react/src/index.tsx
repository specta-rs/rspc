import {
  RSPCError,
  Client,
  ClientFilteredProcs,
  GetProcedure,
  ProcedureKeyTuple,
} from "@rspc/client";
import {
  QueryClientProvider,
  useQuery,
  useMutation,
  hashQueryKey,
} from "@tanstack/react-query";
import type {
  QueryClient,
  UseQueryOptions,
  UseMutationOptions,
} from "@tanstack/react-query";
import * as React from "react";

export interface BaseOptions<TClient extends Client<any, any>> {
  rspc?: {
    client?: TClient;
    abortOnUnmount?: boolean;
  };
}

export interface SubscriptionOptions<TOutput> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

interface Context<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

export function createRspcReact<TClient extends Client<any, any>>() {
  const context = React.createContext<Context<TClient> | null>(null);

  const Provider = ({
    children,
    client,
    queryClient,
  }: React.PropsWithChildren<Context<TClient>>) => (
    <context.Provider
      value={{
        client,
        queryClient,
      }}
    >
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </context.Provider>
  );

  type FilteredProcs = ClientFilteredProcs<TClient>;

  type Queries = FilteredProcs["queries"];
  type Query<K extends Queries["key"]> = GetProcedure<Queries, K>;
  type Mutations = FilteredProcs["mutations"];
  type Mutation<K extends Mutations["key"]> = GetProcedure<Mutations, K>;
  type Subscriptions = FilteredProcs["subscriptions"];
  type Subscription<K extends Subscriptions["key"]> = GetProcedure<
    Subscriptions,
    K
  >;

  type TBaseOptions = BaseOptions<TClient>;

  return {
    Provider,
    createHooks() {
      function useContext() {
        const ctx = React.useContext(context);
        if (!ctx)
          throw new Error(
            "The rspc context has not been set. Ensure you have the `<rspc.Provider>` component higher up in your component tree."
          );
        return ctx;
      }

      return {
        useQuery<K extends Queries["key"]>(
          keyAndInput: ProcedureKeyTuple<K, Query<K>>,
          opts?: Omit<
            UseQueryOptions<
              Query<K>["result"],
              RSPCError,
              Query<K>["result"],
              ProcedureKeyTuple<K, Query<K>>
            >,
            "queryKey" | "queryFn"
          > &
            TBaseOptions
        ) {
          const { rspc, ...rawOpts } = opts ?? {};
          let client = rspc?.client! || useContext().client;

          return useQuery(
            keyAndInput,
            () => client.query(keyAndInput),
            rawOpts
          );
        },
        useMutation<K extends Mutations["key"] & string, TContext = unknown>(
          key: K | [K],
          opts?: UseMutationOptions<
            Mutation<K>["result"],
            RSPCError,
            Mutation<K>["input"] extends null
              ? undefined
              : Mutation<K>["input"],
            TContext
          > &
            TBaseOptions
        ) {
          const { rspc, ...rawOpts } = opts ?? {};
          let client = rspc?.client || useContext().client;

          type Input = Mutation<K>["input"] extends null
            ? undefined
            : Mutation<K>["input"];

          return useMutation(async (input: Input) => {
            const actualKey = Array.isArray(key) ? key[0] : key;
            return client.mutation([actualKey, input] as any);
          }, rawOpts);
        },
        useSubscription<K extends Subscriptions["key"] & string>(
          keyAndInput: ProcedureKeyTuple<K, Subscription<K>>,
          opts: SubscriptionOptions<Subscription<K>["result"]> & TBaseOptions
        ) {
          const enabled = opts?.enabled ?? true;
          const queryKey = hashQueryKey(keyAndInput);
          let client = opts.rspc?.client || useContext().client;

          React.useEffect(() => {
            if (!enabled) {
              return;
            }

            let isStopped = false;

            const subscription = client.subscription(keyAndInput, {
              onStarted: () => {
                if (!isStopped) {
                  opts.onStarted?.();
                }
              },
              onData: (data) => {
                if (!isStopped) {
                  opts.onData(data);
                }
              },
              onError: (err) => {
                if (!isStopped) {
                  opts.onError?.(err);
                }
              },
            });

            return () => {
              isStopped = true;
              subscription.unsubscribe();
            };
          }, [queryKey, enabled]);
        },
      };
    },
  };
}
