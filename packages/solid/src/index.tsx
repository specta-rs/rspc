import {
  JSX,
  createContext,
  useContext as _useContext,
  createEffect,
} from "solid-js";
import {
  Client,
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
  QueryClient,
  CreateQueryOptions,
  CreateQueryResult,
  createQuery as __createQuery,
  createInfiniteQuery as __createInfiniteQuery,
  createMutation as __createMutation,
  CreateInfiniteQueryOptions,
  CreateInfiniteQueryResult,
  CreateMutationOptions,
  CreateMutationResult,
  QueryClientProvider,
  hashQueryKey,
} from "@tanstack/solid-query";

export interface BaseOptions<TProcedures extends ProceduresDef> {
  rspc?: {
    client?: Client<TProcedures>;
  };
}

export interface SubscriptionOptions<TOutput> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

interface Context<TProcedures extends ProceduresDef> {
  client: Client<TProcedures>;
  queryClient: QueryClient;
}

// TODO: The React side is handling types in a whole different way for the normi prototype. Should this be changed to match or should React be rolled back?
// TODO: Also should SolidJS use the hook factory pattern???
export function createSolidQueryHooks<TProceduresLike extends ProceduresDef>() {
  type TProcedures = inferProcedures<TProceduresLike>;
  type TBaseOptions = BaseOptions<TProcedures>;

  const Context = createContext<Context<TProcedures>>(undefined!);

  const Provider = (props: {
    children?: JSX.Element;
    client: { _rspc_def: any }; // TODO: This type is just a slightly safer `as any`. Replace it with proper `Client` type. This will work for now before release.
    queryClient: QueryClient;
  }): JSX.Element => {
    return (
      <Context.Provider
        value={{
          // @ts-expect-error: Bad type for the argument.
          client: props.client,
          queryClient: props.queryClient,
        }}
      >
        <QueryClientProvider client={props.queryClient}>
          {props.children as any}
        </QueryClientProvider>
      </Context.Provider>
    );
  };

  // TODO: This function should require an explicit return type but it's infered as `any` if I don't
  // TODO: Changed this to be typed like the React side.
  function createClient(
    opts: ClientArgs
  ): Client<
    TProcedures,
    TProcedures["queries"],
    TProcedures["mutations"],
    TProcedures["subscriptions"]
  > {
    return _createVanillaClient(opts);
  }

  function useContext() {
    const ctx = _useContext(Context);
    if (ctx?.queryClient === undefined)
      throw new Error(
        "The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree."
      );
    return ctx;
  }

  function createQuery<
    K extends TProcedures["queries"]["key"] & string,
    TQueryFnData = inferQueryResult<TProcedures, K>,
    TData = inferQueryResult<TProcedures, K>
  >(
    keyAndInput: () => [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ],
    opts?: Omit<
      CreateQueryOptions<
        TQueryFnData,
        RSPCError,
        TData,
        () => [K, inferQueryInput<TProcedures, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ): CreateQueryResult<TData, RSPCError> {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __createQuery(
      keyAndInput,
      async () => client!.query(keyAndInput()),
      rawOpts as any
    );
  }

  function createInfiniteQuery<
    K extends inferInfiniteQueries<TProcedures>["key"] & string
  >(
    keyAndInput: () => [
      key: K,
      ...input: _inferInfiniteQueryProcedureHandlerInput<TProcedures, K>
    ],
    opts?: Omit<
      CreateInfiniteQueryOptions<
        inferInfiniteQueryResult<TProcedures, K>,
        RSPCError,
        inferInfiniteQueryResult<TProcedures, K>,
        inferInfiniteQueryResult<TProcedures, K>,
        () => [K, inferQueryInput<TProcedures, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ): CreateInfiniteQueryResult<
    inferInfiniteQueryResult<TProcedures, K>,
    RSPCError
  > {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __createInfiniteQuery(
      keyAndInput,
      async () => {
        throw new Error("TODO: Support infinite query on SolidJS!"); // TODO: Finish this
      },
      rawOpts as any
    );
  }

  function createMutation<
    K extends TProcedures["mutations"]["key"] & string,
    TContext = unknown
  >(
    key: K | [K],
    opts?: CreateMutationOptions<
      inferMutationResult<TProcedures, K>,
      RSPCError,
      inferMutationInput<TProcedures, K> extends null
        ? undefined
        : inferMutationInput<TProcedures, K>,
      TContext
    > &
      TBaseOptions
  ): CreateMutationResult<
    inferMutationResult<TProcedures, K>,
    RSPCError,
    inferMutationInput<TProcedures, K> extends null
      ? undefined
      : inferMutationInput<TProcedures, K>,
    TContext
  > {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __createMutation(async (input) => {
      const actualKey = Array.isArray(key) ? key[0] : key;
      return client!.mutation([actualKey, input] as any);
    }, rawOpts as any);
  }

  function createSubscription<
    K extends TProcedures["subscriptions"]["key"] & string,
    TData = inferSubscriptionResult<TProcedures, K>
  >(
    keyAndInput: () => [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: SubscriptionOptions<TData> & TBaseOptions
  ) {
    let client = opts?.rspc?.client;
    if (!client) {
      client = useContext().client;
    }
    const queryKey = () => hashQueryKey(keyAndInput());
    const enabled = () => opts?.enabled ?? true;

    return createEffect(() => {
      if (!enabled()) {
        return;
      }

      queryKey();

      let isStopped = false;

      const subscription = client!.subscription(keyAndInput(), {
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
    _rspc_def: undefined! as TProceduresLike, // This allows inferring the operations type from TS helpers // TODO: This was removed on React side. Mistake or not?
    createClient,
    useContext,
    Provider,
    createQuery,
    // createInfiniteQuery, // TODO
    createMutation,
    createSubscription,
  };
}
