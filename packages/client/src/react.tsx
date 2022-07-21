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
  useQuery,
} from "@tanstack/react-query";
import { Client, ClientArgs, createClient, OperationsDef } from "./client";

interface Context<T extends OperationsDef> {
  client: Client<T>;
  queryClient: QueryClient;
}

// export type QueryWrapper<T extends OperationsDef> = <
//   K extends T["queries"]["key"]
// >(
//   key: K,
//   arg?: Extract<T["queries"], { key: K }>["arg"],
//   options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
// ) => [
//   any[], // Query Key
//   Extract<T["queries"], { key: K }>["arg"],
//   UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
// ];

export function createReactQueryHooks<T extends OperationsDef>() {
  const Context = React.createContext<Context<T>>(undefined!);
  const ReactQueryContext = React.createContext<QueryClient>(undefined!);

  function useContext() {
    return React.useContext(Context);
  }

  function useQuery<K extends T["queries"]["key"]>(
    key: K,
    arg?: Extract<T["queries"], { key: K }>["arg"],
    options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  ): UseQueryResult<Extract<T["queries"], { key: K }>["result"], unknown> {
    const ctx = useContext();
    return _useQuery([key, arg], async () => ctx.client.query(key, arg), {
      ...options,
      context: ReactQueryContext,
    });
  }

  // function customUseQuery<
  //   K extends T["queries"]["key"]
  //   // Args extends Extract<T["queries"], { key: K }>["arg"]
  // >(
  //   callback: (
  //     key: K,
  //     args: undefined,
  //     options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  //   ) => void
  // ) {
  //   return (
  //     key: K,
  //     arg?: undefined,
  //     options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  //   ) => {
  //     const x = callback(key, arg, options);
  //     // return useQuery();
  //     return {} as any;
  //   };
  // }

  // function customQuery(
  //   callback: <K extends T["queries"]["key"]>(
  //     key: K,
  //     arg?: Extract<T["queries"], { key: K }>["arg"],
  //     options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  //   ) => [
  //     [any],
  //     Extract<T["queries"], { key: K }>["arg"],
  //     UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>,
  //     any
  //   ]
  // ) {
  //   // TODO: Remove duplicate code with the normal useQuery hook above
  //   function useQuery<K extends T["queries"]["key"]>(
  //     key: K,
  //     arg?: Extract<T["queries"], { key: K }>["arg"],
  //     options?: UseQueryOptions<Extract<T["queries"], { key: K }>["result"]>
  //   ): UseQueryResult<Extract<T["queries"], { key: K }>["result"], unknown> {
  //     const ctx = useContext();
  //     const [actualKey, actualArg, actualOptions] = callback(key, arg, options);

  //     return _useQuery(
  //       actualKey,
  //       async () => ctx.client.query(key, actualArg),
  //       {
  //         ...actualOptions,
  //         context: ReactQueryContext,
  //       }
  //     );
  //   }

  //   return useQuery;
  // }

  function useMutation<K extends T["mutations"]["key"]>(
    key: K,
    arg?: Extract<T["mutations"], { key: K }>["arg"], // TODO: Remove this
    options?: UseMutationOptions<Extract<T["mutations"], { key: K }>["result"]>
  ): UseMutationResult<Extract<T["mutations"], { key: K }>["result"], unknown> {
    const ctx = useContext();
    return _useMutation([key, arg], async () => ctx.client.mutation(key, arg), {
      ...options,
      context: ReactQueryContext,
    });
  }

  // function customMutation(
  //   callback: <K extends T["mutations"]["key"]>(
  //     key: K,
  //     arg?: Extract<T["mutations"], { key: K }>["arg"], // TODO: Remove this
  //     options?: UseMutationOptions<
  //       Extract<T["mutations"], { key: K }>["result"]
  //     >
  //   ) => [
  //     [any],
  //     Extract<T["mutations"], { key: K }>["arg"],
  //     UseMutationOptions<Extract<T["mutations"], { key: K }>["result"]>,
  //     any
  //   ]
  // ) {
  //   // TODO: Remove duplicate code with the normal useQuery hook above
  //   function useMutation<K extends T["mutations"]["key"]>(
  //     key: K,
  //     arg?: Extract<T["mutations"], { key: K }>["arg"], // TODO: Remove this
  //     options?: UseMutationOptions<
  //       Extract<T["mutations"], { key: K }>["result"]
  //     >
  //   ): UseMutationResult<
  //     Extract<T["mutations"], { key: K }>["result"],
  //     unknown
  //   > {
  //     const ctx = useContext();
  //     const [actualKey, actualArg, actualOptions] = callback(key, arg, options);

  //     return _useMutation(
  //       actualKey,
  //       async () => ctx.client.mutation(key, actualArg),
  //       {
  //         ...actualOptions,
  //         context: ReactQueryContext,
  //       }
  //     );
  //   }

  //   return useMutation;
  // }

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
