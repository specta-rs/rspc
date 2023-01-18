import {
  JSX,
  createContext,
  useContext as _useContext,
  createEffect,
  Accessor,
} from "solid-js";
import {
  inferInfiniteQueries,
  inferInfiniteQueryResult,
  inferMutationInput,
  inferMutationResult,
  inferProcedures,
  inferQueryInput,
  inferQueryResult,
  inferSubscriptionResult,
  ProceduresDef,
  RSPCError,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  createVanillaClient as _createVanillaClient,
  ClientArgs,
} from "@rspc/client";
import {
  Client,
  ClientFilteredProcs,
  GetProcedure,
  ProcedureKeyTuple,
} from "@rspc/client/src/newClient";
import {
  QueryClient,
  CreateQueryOptions,
  CreateQueryResult,
  createQuery as _createQuery,
  createInfiniteQuery as _createInfiniteQuery,
  createMutation as _createMutation,
  CreateInfiniteQueryOptions,
  CreateInfiniteQueryResult,
  CreateMutationOptions,
  CreateMutationResult,
  QueryClientProvider,
  hashQueryKey,
} from "@tanstack/solid-query";
import { BaseOptions, SubscriptionOptions } from ".";

interface ProviderArgs<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

interface Context<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

// TODO: The React side is handling types in a whole different way for the normi prototype. Should this be changed to match or should React be rolled back?
// TODO: Also should SolidJS use the hook factory pattern???
export function createRspcSolid<TClient extends Client<any, any>>() {
  const context = createContext<Context<TClient>>(undefined!);

  const Provider = ({
    children,
    client,
    queryClient,
  }: ProviderArgs<TClient> & {
    children?: JSX.Element;
  }) => {
    return (
      <context.Provider
        value={{
          client,
          queryClient,
        }}
      >
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </context.Provider>
    );
  };

  type FilteredProcs = ClientFilteredProcs<TClient>;

  type Queries = FilteredProcs["queries"];
  type Query<K extends Queries["key"]> = GetProcedure<Queries, K>;
  type InfiniteQueries = Extract<Queries, { input: { cursor: any } }>;
  type InfiniteQuery<K extends InfiniteQueries["key"]> = Extract<
    InfiniteQueries,
    { key: K }
  >;
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
            "The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree."
          );
        return ctx;
      }

      function createQuery<K extends Queries["key"] & string>(
        keyAndInput: Accessor<ProcedureKeyTuple<Query<K>>>,
        opts?: Omit<
          CreateQueryOptions<
            Query<K>["result"],
            RSPCError,
            Query<K>["result"],
            Accessor<ProcedureKeyTuple<Query<K>>>
          >,
          "queryKey" | "queryFn"
        > &
          TBaseOptions
      ) {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client;
        }

        return _createQuery(
          keyAndInput,
          async () => client!.query(keyAndInput()),
          rawOpts as any
        );
      }

      function createInfiniteQuery<K extends InfiniteQueries["key"] & string>(
        keyAndInput: Accessor<ProcedureKeyTuple<InfiniteQuery<K>>>,
        opts?: Omit<
          CreateInfiniteQueryOptions<
            InfiniteQuery<K>["result"],
            RSPCError,
            InfiniteQuery<K>["result"],
            InfiniteQuery<K>["result"],
            Accessor<ProcedureKeyTuple<InfiniteQuery<K>>>
          >,
          "queryKey" | "queryFn"
        > &
          TBaseOptions
      ) {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client;
        }

        return _createInfiniteQuery(
          keyAndInput,
          async () => {
            throw new Error("TODO: Support infinite query on SolidJS!"); // TODO: Finish this
          },
          rawOpts as any
        );
      }

      function createMutation<
        K extends Mutations["key"] & string,
        TContext = unknown
      >(
        key: K | [K],
        opts?: CreateMutationOptions<
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
          client = useContext().client;
        }

        type Input = Mutation<K>["input"] extends null
          ? undefined
          : Mutation<K>["input"];

        return _createMutation(async (input: Input) => {
          const actualKey = Array.isArray(key) ? key[0] : key;
          return client!.mutation([actualKey, input] as any);
        }, rawOpts);
      }

      function createSubscription<K extends Subscriptions["key"] & string>(
        keyAndInput: Accessor<ProcedureKeyTuple<Subscription<K>>>,
        opts: SubscriptionOptions<Subscription<K>["result"]> & TBaseOptions
      ) {
        const enabled = () => opts?.enabled ?? true;
        const queryKey = () => hashQueryKey(keyAndInput());
        const { client } = useContext();

        return createEffect(() => {
          if (!enabled()) {
            return;
          }

          // Force effect to refresh when key changes
          queryKey();

          let isStopped = false;

          const subscription = client.subscription(keyAndInput(), {
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
        });
      }

      return {
        createQuery,
        createMutation,
        createSubscription,
      };
    },
  };
}
