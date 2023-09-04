"use client";

import * as tanstack from "@tanstack/react-query";
import * as rspc from "@rspc/query-core";
import React from "react";
// TODO: Remove this once off plane

export function createReactQueryHooks<P extends rspc.ProceduresDef>() {
  const Context = React.createContext<rspc.Context<P>>(undefined!);

  const helpers = rspc.createQueryHookHelpers({
    useContext: () => React.useContext(Context),
  });

  type UseQueryOptions<K extends rspc.inferQueries<P>["key"] & string> =
    rspc.HookOptions<
      P,
      rspc.QueryOptionsOmit<
        tanstack.UseQueryOptions<
          rspc.inferQueryResult<P, K>,
          rspc.inferQueryError<P, K>,
          rspc.inferQueryResult<P, K>,
          [K, rspc.inferQueryInput<P, K>]
        >
      >
    >;

  function useQuery<K extends rspc.inferQueries<P>["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: UseQueryOptions<K>
  ) {
    return tanstack.useQuery(helpers.useQueryArgs(keyAndInput, opts));
  }

  type UseMutationOptions<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext = unknown
  > = rspc.HookOptions<
    P,
    tanstack.UseMutationOptions<
      rspc.inferMutationResult<P, K>,
      rspc.inferMutationError<P, K>,
      rspc.inferMutationInput<P, K> extends never
        ? undefined
        : rspc.inferMutationInput<P, K>,
      TContext
    >
  >;

  function useMutation<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext = unknown
  >(key: K | [K], opts?: UseMutationOptions<K, TContext>) {
    return tanstack.useMutation(helpers.useMutationArgs(key, opts));
  }

  function useSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ],
    opts: rspc.HookOptions<
      P,
      rspc.SubscriptionOptions<rspc.inferSubscription<P, K>>
    >
  ) {
    const [client, rawOpts] = helpers.useExtractOps(opts);

    return React.useEffect(
      () =>
        helpers.handleSubscription({
          client,
          keyAndInput,
          opts: rawOpts,
        }),
      [tanstack.hashQueryKey(keyAndInput), opts.enabled ?? true]
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
    }: React.PropsWithChildren<rspc.Context<P>>) => (
      <Context.Provider
        value={{
          client,
          queryClient,
        }}
      >
        <tanstack.QueryClientProvider client={queryClient}>
          {children}
        </tanstack.QueryClientProvider>
      </Context.Provider>
    ),
    useContext: helpers.useContext,
    useQuery,
    // useInfiniteQuery,
    useMutation,
    useSubscription,
  };
}
