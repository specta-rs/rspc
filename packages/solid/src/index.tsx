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

    return __createQuery({
      queryKey: keyAndInput,
      queryFn: async () => client!.query(keyAndInput()),
      ...(rawOpts as any),
    });
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

    return __createInfiniteQuery({
      queryKey: keyAndInput,
      queryFn: async () => {
        throw new Error("TODO"); // TODO: Finish this
      },
      ...(rawOpts as any),
    });
  }

  function createMutation<
    K extends TProcedures["mutations"]["key"] & string,
    TContext = unknown
  >(
    key: K | [K],
    opts?: CreateMutationOptions<
      inferMutationResult<TProcedures, K>,
      RSPCError,
      inferMutationInput<TProcedures, K> extends never
        ? undefined
        : inferMutationInput<TProcedures, K>,
      TContext
    > &
      TBaseOptions
  ): CreateMutationResult<
    inferMutationResult<TProcedures, K>,
    RSPCError,
    inferMutationInput<TProcedures, K> extends never
      ? undefined
      : inferMutationInput<TProcedures, K>,
    TContext
  > {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __createMutation({
      mutationFn: async (input) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client!.mutation([actualKey, input] as any);
      },
      ...(rawOpts as any),
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
    // const queryKey = hashQueryKey(keyAndInput);
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

  // return {
  //   _rspc_def: undefined! as TProceduresLike, // This allows inferring the operations type from TS helpers
  //   Provider: (props: {
  //     children?: JSX.Element;
  //     client: Client<TProcedures>;
  //     queryClient: QueryClient;
  //   }): JSX.Element => {
  //     return (
  //       <Context.Provider
  //         value={{
  //           client: props.client,
  //           queryClient: props.queryClient,
  //         }}
  //       >
  //         <QueryClientProvider_todo client={props.queryClient}>
  //           {props.children as any}
  //         </QueryClientProvider_todo>
  //       </Context.Provider>
  //     ) as any;
  //   },
  //   useContext,
  //   createQuery,
  //   // createInfiniteQuery,
  //   createMutation,
  //   // createSubscription,
  // };
  return undefined as any; // TODO: Fix
}
