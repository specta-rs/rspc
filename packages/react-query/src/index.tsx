"use client";

import {
  createContext,
  ReactElement,
  useContext as _useContext,
  useEffect,
  PropsWithChildren,
} from "react";
import {
  QueryClient,
  UseQueryOptions,
  UseMutationOptions,
  // UseInfiniteQueryResult,
  // UseInfiniteQueryOptions,
  hashQueryKey,
  QueryClientProvider,
} from "@tanstack/react-query";
import * as tanstack from "@tanstack/react-query";
import { AlphaClient, ProceduresDef } from "@rspc/client";
import * as rspc from "@rspc/client";
import {
  SubscriptionOptions,
  BaseOptions,
  Context,
  handleSubscription,
  throwOnError,
  createQueryHookHelpers,
} from "@rspc/query-core";
import React from "react";
// TODO: Remove this once off plane

export function createReactQueryHooks<P extends ProceduresDef>() {
  type TBaseOptions = BaseOptions<P>;

  const Context = createContext<Context<P>>(undefined!);

  const helpers = createQueryHookHelpers({
    useContext: () => React.useContext(Context),
  });

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
    const [client, rawOpts] = helpers.useExtractOps(opts ?? {});

    return tanstack.useQuery({
      queryKey: helpers.mapQueryKey(keyAndInput as any, client),
      queryFn: () => client.query(keyAndInput).then(throwOnError),
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
    const [client, rawOpts] = helpers.useExtractOps(opts ?? {});

    return tanstack.useMutation({
      mutationFn: async (input: any) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client.mutation([actualKey, input] as any).then(throwOnError);
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
    const [client, rawOpts] = helpers.useExtractOps(opts);

    return useEffect(
      () =>
        handleSubscription({
          client,
          keyAndInput,
          opts: rawOpts,
        }),
      [hashQueryKey(keyAndInput), opts.enabled ?? true]
    );
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
    }: PropsWithChildren<Context<P>>) => (
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
    useContext: helpers.useContext,
    useQuery,
    // useInfiniteQuery,
    useMutation,
    useSubscription,
  };
}
