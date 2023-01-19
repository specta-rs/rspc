import {
  RSPCError,
  Client,
  ClientFilteredProcs,
  GetProcedure,
  ProcedureKeyTuple,
} from "@rspc/client";
import {
  QueryClient,
  CreateQueryOptions,
  createQuery,
  createMutation,
  CreateMutationOptions,
  QueryClientProvider,
  hashQueryKey,
} from "@tanstack/solid-query";
import {
  JSX,
  createContext,
  useContext as _useContext,
  createEffect,
  Accessor,
} from "solid-js";

export interface BaseOptions<TClient extends Client<any, any>> {
  rspc?: {
    client?: TClient;
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

export function createRspcSolid<TClient extends Client<any, any>>() {
  const context = createContext<Context<TClient> | null>(null);

  const Provider = ({
    children,
    client,
    queryClient,
  }: Context<TClient> & {
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
        if (!ctx)
          throw new Error(
            "The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree."
          );
        return ctx;
      }

      return {
        createQuery<K extends Queries["key"]>(
          keyAndInput: Accessor<ProcedureKeyTuple<K, Query<K>>>,
          opts?: Omit<
            CreateQueryOptions<
              Query<K>["result"],
              RSPCError,
              Query<K>["result"],
              Accessor<ProcedureKeyTuple<K, Query<K>>>
            >,
            "queryKey" | "queryFn"
          > &
            TBaseOptions
        ) {
          const { rspc, ...rawOpts } = opts ?? {};
          let client = rspc?.client ?? useContext().client;

          return createQuery(
            keyAndInput,
            async () => client.query(keyAndInput()),
            rawOpts as any
          );
        },
        createMutation<K extends Mutations["key"] & string, TContext = unknown>(
          key: K | [K],
          opts?: CreateMutationOptions<
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
          let client = rspc?.client ?? useContext().client;

          type Input = Mutation<K>["input"] extends null
            ? undefined
            : Mutation<K>["input"];

          return createMutation(async (input: Input) => {
            const actualKey = Array.isArray(key) ? key[0] : key;
            return client.mutation([actualKey, input] as any);
          }, rawOpts);
        },
        createSubscription<K extends Subscriptions["key"] & string>(
          keyAndInput: Accessor<ProcedureKeyTuple<K, Subscription<K>>>,
          opts: SubscriptionOptions<Subscription<K>["result"]> & TBaseOptions
        ) {
          const enabled = () => opts?.enabled ?? true;
          const queryKey = () => hashQueryKey(keyAndInput());
          let client = opts.rspc?.client ?? useContext().client;

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
        },
      };
    },
  };
}
