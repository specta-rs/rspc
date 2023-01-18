import { RSPCError } from "@rspc/client";
import {
  Client,
  ClientFilteredProcs,
  GetProcedure,
  ProcedureKeyTuple,
} from "@rspc/client/src/newClient";
import {
  QueryClient,
  QueryClientProvider,
  UseQueryOptions,
  useQuery as _useQuery,
  useMutation as _useMutation,
  UseMutationOptions,
  hashQueryKey,
} from "@tanstack/react-query";
import {
  createContext,
  PropsWithChildren,
  useContext as _useContext,
  useEffect,
} from "react";
import { BaseOptions, SubscriptionOptions } from ".";

interface ProviderArgs<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

interface Context<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

export function createRspcReact<TClient extends Client<any, any>>(_: TClient) {
  const context = createContext<Context<TClient>>(undefined!);

  const Provider = ({
    children,
    client,
    queryClient,
  }: PropsWithChildren<ProviderArgs<TClient>>) => (
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
        const ctx = _useContext(context);
        if (ctx?.queryClient === undefined)
          throw new Error(
            "The rspc context has not been set. Ensure you have the `<rspc.Provider>` component higher up in your component tree."
          );
        return ctx;
      }

      function useQuery<K extends Queries["key"]>(
        keyAndInput: ProcedureKeyTuple<Query<K>>,
        opts?: Omit<
          UseQueryOptions<
            Query<K>["result"],
            RSPCError,
            Query<K>["result"],
            ProcedureKeyTuple<Query<K>>
          >,
          "queryKey" | "queryFn"
        > &
          TBaseOptions
      ) {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client; // TODO: Types
        }

        return _useQuery(
          keyAndInput,
          () => client!.query(keyAndInput),
          rawOpts
        );
      }
      function useMutation<
        K extends Mutations["key"] & string,
        TContext = unknown
      >(
        key: K | [K],
        opts?: UseMutationOptions<
          Mutation<K>["result"],
          RSPCError,
          Mutation<K>["input"] extends null ? undefined : Mutation<K>["input"],
          TContext
        > &
          TBaseOptions
      ) {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client as any; // TODO
        }

        type Input = Mutation<K>["input"] extends null
          ? undefined
          : Mutation<K>["input"];

        return _useMutation(async (input: Input) => {
          const actualKey = Array.isArray(key) ? key[0] : key;
          return client!.mutation([actualKey, input] as any);
        }, rawOpts);
      }
      function useSubscription<K extends Subscriptions["key"] & string>(
        keyAndInput: ProcedureKeyTuple<Subscription<K>>,
        opts: SubscriptionOptions<Subscription<K>["result"]> & TBaseOptions
      ) {
        const enabled = opts?.enabled ?? true;
        const queryKey = hashQueryKey(keyAndInput);
        const { client } = useContext();

        return useEffect(() => {
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
      }

      return {
        useQuery,
        useMutation,
        useSubscription,
      };
    },
  };
}
