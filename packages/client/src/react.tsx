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
import { Client, ClientArgs, createClient, OperationsDef } from "./client";

interface Context<T extends OperationsDef> {
  client: Client<T>;
  queryClient: QueryClient;
}

type A = [string, boolean];

type B = [number];

type C = [...A, ...B];

export function createReactQueryHooks<T extends OperationsDef>() {
  const Context = React.createContext<Context<T>>(undefined!);
  const ReactQueryContext = React.createContext<QueryClient>(undefined!);

  function useContext() {
    return React.useContext(Context);
  }

  function useQuery<K extends T["queries"]["key"]>(
    key: K,
    options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  ): UseQueryResult<Extract<T["queries"], { key: K }>["result"], unknown> {
    const ctx = useContext();
    return _useQuery(key, async () => ctx.client.query(key), {
      ...options,
      context: ReactQueryContext,
    });
  }

  function useMutation<K extends T["mutations"]["key"]>(
    key: K[0],
    options?: UseMutationOptions<Extract<T["mutations"], { key: K }>["result"]>
  ): UseMutationResult<
    Extract<T["mutations"], { key: K }>["result"],
    unknown,
    K[1]
  > {
    const ctx = useContext();
    return _useMutation(async (data) => ctx.client.mutation([key, data]), {
      ...options,
      context: ReactQueryContext,
    });
  }

  function useSubscription<K extends T["subscriptions"]["key"]>(
    key: K,
    options?: {
      onNext(msg: Extract<T["subscriptions"], { key: K }>["result"]);
      onError(err: never); // TODO: Error type??
    }
  ) {
    const ctx = useContext();
    useEffect(() => {
      ctx.client.addSubscription(key, options);

      return () => {
        // TODO: Handle unsubscribe
      };
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
    // customUseQuery,
    // customQuery,
    useMutation,
    // customMutation,
    useSubscription,
    // useDehydratedState,
    // useInfiniteQuery,
  };
}
