import React, { ReactElement, useEffect } from "react";
import {
  QueryClient,
  useQuery as _useQuery,
  useMutation as _useMutation,
  UseQueryResult,
  UseQueryOptions,
  UseMutationResult,
  UseMutationOptions,
  QueryClientProvider,
} from "@tanstack/react-query";
import { Client, Procedures, RSPCError } from "@rspc/client";

// TODO: Delete these helpers or move them to "./typescript"
// TODO: Allow passing `{ rspc: { client: ... } }` to override the client -> Like `@rspc/solid` supports

export type OperationKeyArgs<
  Operations extends Procedures,
  Type extends keyof Procedures,
  K extends Operations[Type]["key"][0]
> = Extract<
  Operations[Type],
  { key: [K] | [K, any] }
>["key"][1] extends undefined
  ? [K]
  : [K, Extract<Operations[Type], { key: [K, any] }>["key"][1]];

export type OperationKeyResult<
  Operations extends Procedures,
  Type extends keyof Procedures,
  K extends Operations[Type]["key"][0]
> = Extract<
  Operations[Type],
  { key: [K] | [K, any] }
>["key"][1] extends undefined
  ? [K]
  : [K, Extract<Operations[Type], { key: [K, any] }>["key"][1]];

export type Demo<
  Operations extends Procedures,
  Type extends keyof Procedures,
  K
> = Extract<Operations[Type], { key: [K] | [K, any] }>;

interface Context<T extends Procedures> {
  client: Client<T>;
  queryClient: QueryClient;
}

export function createReactQueryHooks<Operations extends Procedures>() {
  const Context = React.createContext<Context<Operations>>(undefined!);
  const ReactQueryContext = React.createContext<QueryClient>(undefined!);

  function useRspcContext() {
    return React.useContext(Context);
  }

  function useQuery<K extends Operations["queries"]["key"][0]>(
    key: Demo<Operations, "queries", K>["key"],
    options?: UseQueryOptions<
      Demo<Operations, "queries", K>["result"],
      RSPCError
    >
  ): UseQueryResult<Demo<Operations, "queries", K>["result"], RSPCError> {
    const ctx = useRspcContext();
    return _useQuery(key, async () => ctx.client.query(key), {
      ...options,
      context: ReactQueryContext,
    });
  }

  function useMutation<K extends Operations["mutations"]["key"]>(
    key: K[0],
    options?: UseMutationOptions<
      Extract<Operations["mutations"], { key: K }>["result"],
      RSPCError,
      K[1]
    >
  ): UseMutationResult<
    Extract<Operations["mutations"], { key: K }>["result"],
    RSPCError,
    K[1]
  > {
    const ctx = useRspcContext();
    return _useMutation(async (data) => ctx.client.mutation([key, data]), {
      ...options,
      context: ReactQueryContext,
    });
  }

  type SubscriptionKey = Operations["subscriptions"]["key"][0];
  type SubscriptionArg<K extends string> = Extract<
    Operations["subscriptions"],
    { key: [K] | [K, any] }
  >["key"][1];
  type SubscriptionResult<K extends string> = Extract<
    Operations["subscriptions"],
    { key: [K] | [K, any] }
  >["result"];

  function useSubscription<K extends SubscriptionKey>(
    key: SubscriptionArg<K> extends undefined ? [K] : [K, SubscriptionArg<K>],
    options?: {
      onNext(msg: SubscriptionResult<K>);
      onError?(err: RSPCError);
    }
  ) {
    const ctx = useRspcContext();

    useEffect(() => {
      const unsub = ctx.client.addSubscription(key, options);
      return () => unsub();
    }, []);
  }

  return {
    _rspc_def: undefined as Operations, // This allows inferring the operations type from TS helpers
    Provider: ({
      children,
      client,
      queryClient,
    }: {
      children?: ReactElement;
      client: Client<Operations>;
      queryClient: QueryClient;
    }) => (
      <Context.Provider
        value={{
          client,
          queryClient,
        }}
      >
        <ReactQueryContext.Provider value={queryClient}>
          <QueryClientProvider client={queryClient}>
            {children}
          </QueryClientProvider>
        </ReactQueryContext.Provider>
      </Context.Provider>
    ),
    useContext: useRspcContext,
    useQuery,
    useMutation,
    useSubscription,
    // useDehydratedState, // TODO
    // useInfiniteQuery, // TODO
    ReactQueryContext,
  };
}
