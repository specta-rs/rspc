"use client";

import {
  createContext,
  ReactElement,
  useContext as _useContext,
  useEffect,
} from "react";
import {
  QueryClient,
  UseQueryOptions,
  UseMutationOptions,
  UseInfiniteQueryResult,
  UseInfiniteQueryOptions,
  hashQueryKey,
  QueryClientProvider,
} from "@tanstack/react-query";
import * as tanstack from "@tanstack/react-query";
import { AlphaClient, ProceduresDef, ProcedureDef } from "@rspc/client";
import * as rspc from "@rspc/client";

// TODO: Remove this once off plane
export { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// TODO: Reuse one from client but don't export it in public API
type KeyAndInput = [string] | [string, any];

export interface BaseOptions<TProcedures extends ProceduresDef> {
  rspc?: {
    client?: AlphaClient<TProcedures>;
  };
}

export interface SubscriptionOptions<P extends ProcedureDef> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: P["result"]) => void;
  onError?: (err: P["error"] | rspc.Error) => void;
}

export interface Context<TProcedures extends ProceduresDef> {
  client: AlphaClient<TProcedures>;
  queryClient: QueryClient;
}

// TODO: Share with SolidJS hooks if possible?
export type HooksOpts<P extends ProceduresDef> = {
  context: React.Context<Context<P>>;
};

export function createReactQueryHooks<P extends ProceduresDef>() {
  type TBaseOptions = BaseOptions<P>;

  const mapQueryKey: (
    keyAndInput: KeyAndInput,
    client: AlphaClient<P>
  ) => KeyAndInput = (keyAndInput, client) =>
    (client as any).mapQueryKey?.(keyAndInput) || keyAndInput;

  const Context = createContext<Context<P>>(undefined!);

  function useContext() {
    const ctx = _useContext(Context);
    if (ctx?.queryClient === undefined)
      throw new Error(
        "The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree."
      );
    return ctx;
  }

  function useQuery<K extends rspc.inferQueries<P>["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: Omit<
      UseQueryOptions<
        rspc.inferQueryResult<P, K>,
        rspc.inferQueryError<P, K>,
        rspc.inferQueryResult<P, K>,
        [K, rspc.inferQueryInput<P, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ) {
    const { rspc, ...rawOpts } = opts ?? {};

    const client = opts?.rspc?.client ?? useContext().client;

    return tanstack.useQuery({
      queryKey: mapQueryKey(keyAndInput as any, client) as any,
      queryFn: () =>
        client.query(keyAndInput).then((res) => {
          if (res.status === "ok") return res.data;
          else throw res.error;
        }),
      ...rawOpts,
    });
  }

  function useMutation<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext = unknown
  >(
    key: K | [K],
    opts?: UseMutationOptions<
      rspc.inferMutationResult<P, K>,
      rspc.inferMutationError<P, K>,
      rspc.inferMutationInput<P, K> extends never
        ? undefined
        : rspc.inferMutationInput<P, K>,
      TContext
    > &
      TBaseOptions
  ) {
    const { rspc, ...rawOpts } = opts ?? {};

    const client = opts?.rspc?.client ?? useContext().client;

    return tanstack.useMutation({
      mutationFn: async (input: any) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client.mutation([actualKey, input] as any).then((res) => {
          if (res.status === "ok") return res.data;
          else return Promise.reject(res.error);
        });
      },
      ...rawOpts,
    });
  }

  function useSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ],
    opts: SubscriptionOptions<rspc.inferSubscription<P, K>> & TBaseOptions
  ) {
    let client = opts?.rspc?.client ?? useContext().client;

    const queryKey = hashQueryKey(keyAndInput);
    const enabled = opts?.enabled ?? true;

    let isStopped = false;

    return useEffect(() => {
      if (!enabled) return;
      const unsubscribe = client.addSubscription<K>(keyAndInput, {
        onData: (data) => {
          if (!isStopped) opts.onData(data);
        },
        onError: (error) => {
          if (!isStopped) opts.onError?.(error);
        },
      });

      return () => {
        isStopped = true;
        unsubscribe();
      };
    }, [queryKey, enabled]);
  }

  // function useInfiniteQuery<K extends inferInfiniteQueries<P>["key"] & string>(
  //   keyAndInput: [
  //     key: K,
  //     ...input: _inferInfiniteQueryProcedureHandlerInput<P, K>
  //   ],
  //   opts?: Omit<
  //     UseInfiniteQueryOptions<
  //       inferInfiniteQueryResult<P, K>,
  //       RSPCError,
  //       inferInfiniteQueryResult<P, K>,
  //       inferInfiniteQueryResult<P, K>,
  //       [K, inferQueryInput<P, K>]
  //     >,
  //     "queryKey" | "queryFn"
  //   > &
  //     TBaseOptions
  // ): UseInfiniteQueryResult<inferInfiniteQueryResult<P, K>, RSPCError> {
  //   const { rspc, ...rawOpts } = opts ?? {};
  //   let client = rspc?.client;
  //   if (!client) {
  //     client = useContext().client;
  //   }

  //   return __useInfiniteQuery({
  //     queryKey: mapQueryKey(keyAndInput as any),
  //     queryFn: async () => {
  //       throw new Error("TODO"); // TODO: Finish this
  //     },
  //     ...(rawOpts as any),
  //   });
  // }

  return {
    _rspc_def: undefined! as P, // This allows inferring the operations type from TS helpers
    Provider: ({
      children,
      client,
      queryClient,
    }: {
      children?: ReactElement;
      client: AlphaClient<P>;
      // TODO: Remove this arg and infer from React context?
      queryClient: QueryClient;
    }) => (
      <Context.Provider
        value={{
          client,
          queryClient,
        }}
      >
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </Context.Provider>
    ),
    useContext,
    useQuery,
    // useInfiniteQuery,
    useMutation,
    useSubscription,
  };
}
