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
import { Client, OperationsDef } from "./client";
import { RSPCError } from "./error";

export type OperationKeyArgs<
  Operations extends OperationsDef,
  Type extends keyof OperationsDef,
  K extends Operations[Type]["key"][0]
> = Extract<
  Operations[Type],
  { key: [K] | [K, any] }
>["key"][1] extends undefined
  ? [K]
  : [K, Extract<Operations[Type], { key: [K, any] }>["key"][1]];

export type OperationKeyResult<
  Operations extends OperationsDef,
  Type extends keyof OperationsDef,
  K extends Operations[Type]["key"][0]
> = Extract<
  Operations[Type],
  { key: [K] | [K, any] }
>["key"][1] extends undefined
  ? [K]
  : [K, Extract<Operations[Type], { key: [K, any] }>["key"][1]];

export type Demo<
  Operations extends OperationsDef,
  Type extends keyof OperationsDef,
  K
> = Extract<Operations[Type], { key: [K] | [K, any] }>;

interface Context<T extends OperationsDef> {
  client: Client<T>;
  queryClient: QueryClient;
}

export function createReactQueryHooks<T extends OperationsDef>() {
  const Context = React.createContext<Context<T>>(undefined!);
  const ReactQueryContext = React.createContext<QueryClient>(undefined!);

  function useContext() {
    return React.useContext(Context);
  }

  function useQuery<K extends T["queries"]["key"][0]>(
    key: Demo<T, "queries", K>["key"],
    options?: UseQueryOptions<Demo<T, "queries", K>["result"], RSPCError>
  ): UseQueryResult<Demo<T, "queries", K>["result"], RSPCError> {
    const ctx = useContext();
    return _useQuery(key, async () => ctx.client.query(key), {
      ...options,
      context: ReactQueryContext,
    });
  }

  function useMutation<K extends T["mutations"]["key"]>(
    key: K[0],
    options?: UseMutationOptions<
      Extract<T["mutations"], { key: K }>["result"],
      RSPCError,
      K[1]
    >
  ): UseMutationResult<
    Extract<T["mutations"], { key: K }>["result"],
    RSPCError,
    K[1]
  > {
    const ctx = useContext();
    return _useMutation(async (data) => ctx.client.mutation([key, data]), {
      ...options,
      context: ReactQueryContext,
    });
  }

  type SubscriptionKey = T["subscriptions"]["key"][0];
  type SubscriptionArg<K extends string> = Extract<
    T["subscriptions"],
    { key: [K] | [K, any] }
  >["key"][1];
  type SubscriptionResult<K extends string> = Extract<
    T["subscriptions"],
    { key: [K] | [K, any] }
  >["result"];

  function useSubscription<K extends SubscriptionKey>(
    key: SubscriptionArg<K> extends undefined ? [K] : [K, SubscriptionArg<K>],
    options?: {
      onNext(msg: SubscriptionResult<K>);
      onError?(err: RSPCError);
    }
  ) {
    const ctx = useContext();

    useEffect(() => {
      const unsub = ctx.client.addSubscription(key, options);
      return () => unsub();
    }, []);
  }

  return {
    Provider: ({
      children,
      client,
      queryClient,
    }: {
      children: ReactElement;
      client: Client<T>;
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
    useContext: () => {
      return React.useContext(Context);
    },
    useQuery,
    useMutation,
    useSubscription,
    // useDehydratedState,
    // useInfiniteQuery,
  };
}
