import { JSX, createContext, useContext as _useContext } from "solid-js";
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
  ProceduresLike,
} from "@rspc/client";
import {
  QueryClient,
  CreateQueryOptions,
  CreateQueryResult,
  createQuery as __createQuery,
  createInfiniteQuery as __createInfiniteQuery,
  createMutation as __createMutation,
  CreateInfiniteQueryResult,
  CreateMutationResult,
  QueryClientProvider,
  SolidQueryOptions,
  SolidInfiniteQueryOptions,
  SolidMutationOptions,
  FetchQueryOptions,
} from "@tanstack/solid-query";

type FunctionedParams<T> = () => T;

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

export function createSolidQueryHooks<TProceduresLike extends ProceduresDef>() {
  type TProcedures = inferProcedures<TProceduresLike>;
  type TBaseOptions = BaseOptions<TProcedures>;

  const Context = createContext<Context<TProcedures>>(undefined!);

  function useContext() {
    const ctx = _useContext(Context);
    if (ctx?.queryClient === undefined)
      throw new Error(
        "The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree."
      );
    return ctx;
  }

  function useUtils() {
    return createRSPCQueryUtils<TProceduresLike>(useContext());
  }

  function createQuery<
    K extends TProcedures["queries"]["key"] & string,
    TQueryFnData = inferQueryResult<TProcedures, K>,
    TData = inferQueryResult<TProcedures, K>
  >(
    opts?: FunctionedParams<
      {
        queryKey: [
          key: K,
          ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
        ];
      } & Omit<
        SolidQueryOptions<
          TQueryFnData,
          RSPCError,
          TData,
          [K, inferQueryInput<TProcedures, K>]
        >,
        "queryKey" | "queryFn"
      > &
        TBaseOptions
    >
  ): CreateQueryResult<TData, RSPCError> {
    return __createQuery(() => {
      const { rspc, queryKey, ...rawOpts } = opts?.() ?? {};
      let client = rspc?.client;
      if (!client) {
        client = useContext().client;
      }

      return {
        queryKey: queryKey,
        queryFn: async () => {
          return client!.query(queryKey as any);
        },
        ...(rawOpts as any),
      };
    });
  }

  function createInfiniteQuery<
    K extends inferInfiniteQueries<TProcedures>["key"] & string
  >(
    opts?: FunctionedParams<
      {
        queryKey: [
          key: K,
          ...input: _inferInfiniteQueryProcedureHandlerInput<TProcedures, K>
        ];
      } & Omit<
        SolidInfiniteQueryOptions<
          inferInfiniteQueryResult<TProcedures, K>,
          RSPCError,
          inferInfiniteQueryResult<TProcedures, K>,
          inferInfiniteQueryResult<TProcedures, K>,
          [K, inferQueryInput<TProcedures, K>]
        >,
        "queryKey" | "queryFn"
      > &
        TBaseOptions
    >
  ): CreateInfiniteQueryResult<
    inferInfiniteQueryResult<TProcedures, K>,
    RSPCError
  > {
    return __createInfiniteQuery(() => {
      const { rspc, queryKey, ...rawOpts } = opts?.() ?? {};
      let client = rspc?.client;
      if (!client) {
        client = useContext().client;
      }

      return {
        queryKey: queryKey,
        queryFn: async () => {
          throw new Error("TODO"); // TODO: Finish this
        },
        ...(rawOpts as any),
      };
    });
  }

  function createMutation<
    K extends TProcedures["mutations"]["key"] & string,
    TContext = unknown
  >(
    opts?: FunctionedParams<
      {
        mutationKey: K | [K];
      } & Omit<
        SolidMutationOptions<
          inferMutationResult<TProcedures, K>,
          RSPCError,
          inferMutationInput<TProcedures, K> extends never
            ? undefined
            : inferMutationInput<TProcedures, K>,
          TContext
        >,
        "mutationKey"
      > &
        TBaseOptions
    >
  ): CreateMutationResult<
    inferMutationResult<TProcedures, K>,
    RSPCError,
    inferMutationInput<TProcedures, K> extends never
      ? undefined
      : inferMutationInput<TProcedures, K>,
    TContext
  > {
    return __createMutation(() => {
      const { rspc, mutationKey, ...rawOpts } = opts?.() ?? {};
      let client = rspc?.client;
      if (!client) {
        client = useContext().client;
      }

      return {
        mutationKey,
        mutationFn: async (input) => {
          const actualKey = Array.isArray(mutationKey)
            ? mutationKey[0]
            : mutationKey;
          return client!.mutation([actualKey, input] as any);
        },
        ...(rawOpts as any),
      };
    });
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
    // const queryKey = hashKey(keyAndInput);
    // const enabled = opts?.enabled ?? true;

    throw new Error("TODO: SolidJS Subscriptions are not supported yet!");

    // return useEffect(() => {
    //   if (!enabled) {
    //     return;
    //   }
    //   let isStopped = false;
    //   const unsubscribe = client.addSubscription<K, TData>(
    //     keyAndInput,
    //     {
    //       onStarted: () => {
    //         if (!isStopped) {
    //           opts.onStarted?.();
    //         }
    //       },
    //       onData: (data) => {
    //         if (!isStopped) {
    //           opts.onData(data);
    //         }
    //       },
    //       onError: (err) => {
    //         if (!isStopped) {
    //           opts.onError?.(err);
    //         }
    //       },
    //     }
    //   );
    //   return () => {
    //     isStopped = true;
    //     unsubscribe();
    //   };
    // }, [queryKey, enabled]);
  }

  return {
    _rspc_def: undefined! as TProceduresLike, // This allows inferring the operations type from TS helpers
    Provider: (props: {
      children?: JSX.Element;
      client: Client<TProcedures>;
      queryClient: QueryClient;
    }): JSX.Element => {
      return (
        <Context.Provider
          value={{
            client: props.client,
            queryClient: props.queryClient,
          }}
        >
          <QueryClientProvider client={props.queryClient}>
            {props.children as any}
          </QueryClientProvider>
        </Context.Provider>
      ) as any;
    },
    useContext,
    useUtils,
    createQuery,
    // createInfiniteQuery,
    createMutation,
    // createSubscription,
  };
}

function createRSPCQueryUtils<TProceduresLike extends ProceduresDef>(
  context: Context<inferProcedures<TProceduresLike>>
) {
  type TProcedures = inferProcedures<TProceduresLike>;
  type TBaseOptions = BaseOptions<TProcedures>;
  type QueryKey<K extends string, TProcedures extends ProceduresLike> = [
    key: K,
    ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
  ];
  type AllowedKeys = TProcedures["queries"]["key"] & string;
  type QueryOptions<
    K extends string,
    TData,
    TProcedures extends ProceduresLike
  > = Omit<
    FetchQueryOptions<TData, RSPCError, TData, QueryKey<K, TProcedures>>,
    "queryKey" | "queryFn"
  > &
    TBaseOptions;

  const queryClient = context.queryClient;
  const client = context.client;

  function fetchQuery<
    K extends AllowedKeys,
    TData = inferQueryResult<TProcedures, K>
  >(
    queryKey: QueryKey<K, TProcedures>,
    opts?: QueryOptions<K, TData, TProcedures>
  ): Promise<TData> {
    return queryClient.fetchQuery({
      queryKey: queryKey,
      queryFn: async () => {
        return client.query(queryKey as any);
      },
      ...(opts as any),
    });
  }

  function prefetchQuery<
    K extends AllowedKeys,
    TData = inferQueryResult<TProcedures, K>
  >(
    queryKey: QueryKey<K, TProcedures>,
    opts?: QueryOptions<K, TData, TProcedures>
  ): Promise<void> {
    return queryClient.prefetchQuery({
      queryKey: queryKey,
      queryFn: async () => {
        return client.query(queryKey as any);
      },
      ...(opts as any),
    });
  }

  return {
    fetchQuery,
    prefetchQuery,
  };
}
