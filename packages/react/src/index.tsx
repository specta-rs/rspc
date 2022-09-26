import {
  createContext,
  ReactElement,
  useContext as _useContext,
  useEffect,
} from "react";
import {
  QueryClient,
  useQuery as __useQuery,
  useInfiniteQuery as __useInfiniteQuery,
  useMutation as __useMutation,
  UseQueryResult,
  UseQueryOptions,
  UseMutationResult,
  UseMutationOptions,
  UseInfiniteQueryResult,
  UseInfiniteQueryOptions,
  hashQueryKey,
} from "@tanstack/react-query";
import {
  Client,
  ProceduresLike,
  RSPCError,
  inferProcedures,
  _inferProcedureHandlerInput,
  inferInfiniteQueries,
  _inferInfiniteQueryProcedureHandlerInput,
  inferInfiniteQueryResult,
} from "@rspc/client";
import { inferQueryInput } from "@rspc/client";
import { inferQueryResult } from "@rspc/client";
import { inferMutationResult } from "@rspc/client";
import { inferMutationInput } from "@rspc/client";
import { inferSubscriptionResult } from "@rspc/client";
import { ProceduresDef } from "@rspc/client";

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

export interface Context<TProcedures extends ProceduresDef> {
  client: Client<TProcedures>;
  queryClient: QueryClient;
}

export function createReactQueryHooks<
  TProceduresLike extends ProceduresLike
>() {
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

  function useQuery<
    K extends TProcedures["queries"]["key"] & string,
    TQueryFnData = inferQueryResult<TProcedures, K>,
    TData = inferQueryResult<TProcedures, K>
  >(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ],
    opts?: Omit<
      UseQueryOptions<
        TQueryFnData,
        RSPCError,
        TData,
        [K, inferQueryInput<TProcedures, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ): UseQueryResult<TData, RSPCError> {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __useQuery(
      keyAndInput,
      async () => {
        return await client!.query(keyAndInput[0], keyAndInput[1]);
      },
      rawOpts as any
    );
  }

  function useInfiniteQuery<
    K extends inferInfiniteQueries<TProcedures>["key"] & string
  >(
    keyAndInput: [
      key: K,
      ...input: _inferInfiniteQueryProcedureHandlerInput<TProcedures, K>
    ],
    opts?: Omit<
      UseInfiniteQueryOptions<
        inferInfiniteQueryResult<TProcedures, K>,
        RSPCError,
        inferInfiniteQueryResult<TProcedures, K>,
        inferInfiniteQueryResult<TProcedures, K>,
        [K, inferQueryInput<TProcedures, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ): UseInfiniteQueryResult<
    inferInfiniteQueryResult<TProcedures, K>,
    RSPCError
  > {
    const { rspc, ...rawOpts } = opts ?? {};
    let client = rspc?.client;
    if (!client) {
      client = useContext().client;
    }

    return __useInfiniteQuery(
      keyAndInput,
      async () => {
        throw new Error("TODO"); // TODO: Finish this
      },
      rawOpts as any
    );
  }

  function useMutation<
    K extends TProcedures["mutations"]["key"] & string,
    TContext = unknown
  >(
    key: K | [K],
    opts?: UseMutationOptions<
      inferMutationResult<TProcedures, K>,
      RSPCError,
      inferMutationInput<TProcedures, K> extends never
        ? undefined
        : inferMutationInput<TProcedures, K>,
      TContext
    > &
      TBaseOptions
  ): UseMutationResult<
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

    return __useMutation(async (input) => {
      const actualKey = Array.isArray(key) ? key[0] : key;
      return client!.mutation(actualKey, input);
    }, rawOpts as any);
  }

  function useSubscription<
    K extends TProcedures["subscriptions"]["key"] & string,
    TData = inferSubscriptionResult<TProcedures, K>
  >(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: SubscriptionOptions<TData> & TBaseOptions
  ) {
    let client = opts?.rspc?.client;
    if (!client) {
      client = useContext().client;
    }
    const queryKey = hashQueryKey(keyAndInput);

    const enabled = opts?.enabled ?? true;

    return useEffect(() => {
      if (!enabled) {
        return;
      }
      let isStopped = false;
      const unsubscribe = client!.addSubscription<K, TData>(
        keyAndInput[0],
        keyAndInput[1],
        {
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
        }
      );
      return () => {
        isStopped = true;
        unsubscribe();
      };
    }, [queryKey, enabled]);
  }

  return {
    _rspc_def: undefined! as TProcedures, // This allows inferring the operations type from TS helpers
    Provider: ({
      children,
      client,
      queryClient,
    }: {
      children?: ReactElement;
      client: Client<TProcedures>;
      queryClient: QueryClient;
    }) => (
      <Context.Provider
        value={{
          client,
          queryClient,
        }}
      >
        {children}
      </Context.Provider>
    ),
    useContext,
    useQuery,
    useInfiniteQuery,
    useMutation,
    useSubscription,
  };
}
