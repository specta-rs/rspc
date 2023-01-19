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
} from "@tanstack/solid-query";
import {
  JSX,
  createContext,
  useContext as _useContext,
  createEffect,
  Accessor,
  onCleanup,
  splitProps,
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

interface ContextType<TClient extends Client<any, any>> {
  queryClient: QueryClient;
  client: TClient;
}

export function createRspcSolid<TClient extends Client<any, any>>() {
  const Context = createContext<ContextType<TClient> | null>(null);

  const Provider = (
    props: ContextType<TClient> & {
      children?: JSX.Element;
    }
  ) => {
    const [ctx, others] = splitProps(props, ["client", "queryClient"]);
    console.log("PROVIDER!!!");
    return (
      <Context.Provider value={ctx}>
        <QueryClientProvider client={ctx.queryClient}>
          {others.children}
        </QueryClientProvider>
      </Context.Provider>
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
        const ctx = _useContext(Context);
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
              Query<K>["result"]
              // opts needs to be typed, and this prevents it
              // Accessor<ProcedureKeyTuple<K, Query<K>>>
            >,
            "queryKey" | "queryFn"
          > &
            TBaseOptions
        ) {
          const { rspc, ...rawOpts } = opts ?? {};
          let client = rspc?.client ?? useContext().client;

          return createQuery(
            keyAndInput as any,
            () => client.query(keyAndInput()),
            rawOpts
          );
        },
        createMutation<K extends Mutations["key"], TContext = unknown>(
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
        createSubscription<K extends Subscriptions["key"]>(
          keyAndInput: Accessor<ProcedureKeyTuple<K, Subscription<K>>>,
          opts: SubscriptionOptions<Subscription<K>["result"]> & TBaseOptions
        ) {
          const enabled = () => opts?.enabled ?? true;
          let client = opts.rspc?.client ?? useContext().client;

          createEffect(() => {
            if (!enabled()) {
              return;
            }

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

            onCleanup(() => {
              isStopped = true;
              subscription.unsubscribe();
            });
          });
        },
      };
    },
  };
}
