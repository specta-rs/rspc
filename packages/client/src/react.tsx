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

  function demo() {
    useQuery(["version", "string"]);
  }

  function useQuery<K extends T["queries"]["key"]>(
    key: [
      ...K,
      ...(Extract<T["queries"], { key: K }>["margs"] extends null ? [] : [K])
    ],
    options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  ): UseQueryResult<Extract<T["queries"], { key: K }>["result"], unknown> {
    const ctx = useContext();
    return _useQuery(
      key,
      async () => ctx.client.query([key[0], key[1]], key[2]),
      {
        ...options,
        context: ReactQueryContext,
      }
    );
  }

  function useMutation<K extends T["mutations"]["key"]>(
    key: K[0],
    options?: UseMutationOptions<Extract<T["mutations"], { key: K }>["result"]>
  ): UseMutationResult<Extract<T["mutations"], { key: K }>["result"], unknown> {
    const ctx = useContext();
    return _useMutation(
      async (data: K[2] extends undefined ? [K[1]] : [K[1], K[2]]) =>
        ctx.client.mutation([key, data[0], data[1]]),
      {
        ...options,
        context: ReactQueryContext,
      }
    );
  }

  function useSubscription<K extends T["subscriptions"]["key"]>(
    key: K,
    arg?: Extract<T["queries"], { key: K }>["arg"],
    options?: {
      onNext(msg: Extract<T["queries"], { key: K }>["result"]);
      onError(err: never); // TODO: Error type??
    }
  ) {
    useEffect(() => {
      this.transport.subscribe(
        "subscriptionAdd",
        key,
        arg,
        options?.onNext as any,
        options?.onError as any
      );

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
