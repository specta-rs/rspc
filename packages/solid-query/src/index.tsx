/** @jsxImportSource solid-js */

import * as Solid from "solid-js";
import * as tanstack from "@tanstack/solid-query";
import * as rspc from "@rspc/query-core";

export function createRSPCSolidQuery<P extends rspc.ProceduresDef>(
  client: rspc.Client<P>
) {
  return createRawRSPCSolidQuery({ root: client._root });
}

export function createRawRSPCSolidQuery<P extends rspc.ProceduresDef>(_: {
  root: rspc.Root<P>;
}) {
  const Context = Solid.createContext<rspc.Context<P> | null>(null);

  const helpers = rspc.createQueryHookHelpers({
    useContext: () => Solid.useContext(Context),
  });

  type CreateQueryOptions<K extends rspc.inferQueries<P>["key"] & string> =
    rspc.HookOptions<
      P,
      rspc.QueryOptionsOmit<
        tanstack.SolidQueryOptions<
          rspc.inferQueryResult<P, K>,
          rspc.inferQueryError<P, K>,
          rspc.inferQueryResult<P, K>,
          [K, rspc.inferQueryInput<P, K>]
        >
      >
    >;

  function createQuery<K extends rspc.inferQueries<P>["key"] & string>(
    keyAndInput: () => [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: CreateQueryOptions<K>
  ) {
    return tanstack.createQuery(() =>
      helpers.useQueryArgs(keyAndInput(), opts)
    );
  }

  type CreateMutationOptions<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext
  > = rspc.HookOptions<
    P,
    tanstack.SolidMutationOptions<
      rspc.inferMutationResult<P, K>,
      rspc.inferMutationError<P, K>,
      rspc.inferMutationInput<P, K> extends never
        ? undefined
        : rspc.inferMutationInput<P, K>,
      TContext
    >
  >;

  function createMutation<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext = unknown
  >(key: K | [K], opts?: CreateMutationOptions<K, TContext>) {
    return tanstack.createMutation(() => helpers.useMutationArgs(key, opts));
  }

  function createSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >(
    keyAndInput: () => [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ],
    opts: () => rspc.HookOptions<
      P,
      rspc.SubscriptionOptions<rspc.inferSubscription<P, K>>
    >
  ) {
    return Solid.createEffect(
      Solid.on(
        () => [keyAndInput(), opts?.()] as const,
        ([keyAndInput, opts]) => {
          const [client, rawOpts] = helpers.useExtractOps(opts ?? {});

          // TODO: solid-start useRequest

          const cleanup = helpers.handleSubscription({
            client,
            keyAndInput,
            opts: rawOpts,
          });

          Solid.onCleanup(() => cleanup?.());
        }
      )
    );
  }

  // function createInfiniteQuery<
  //   K extends rspc.inferInfiniteQueries<P>["key"] & string
  // >(
  //   keyAndInput: () => [
  //     key: K,
  //     ...input: Omit<
  //       rspc._inferInfiniteQueryProcedureHandlerInput<P, K>,
  //       "cursor"
  //     >
  //   ],
  //   opts?: Omit<
  //     tanstack.CreateInfiniteQueryOptions<
  //       rspc.inferInfiniteQueryResult<P, K>,
  //       rspc.inferInfiniteQueryError<P, K>,
  //       rspc.inferInfiniteQueryResult<P, K>,
  //       rspc.inferInfiniteQueryResult<P, K>,
  //       () => [K, Omit<rspc.inferQueryInput<P, K>, "cursor">]
  //     >,
  //     "queryKey" | "queryFn"
  //   > &
  //     TBaseOptions
  // ) {
  //   const { rspc, ...rawOpts } = opts ?? {};
  //   let client = rspc?.client;
  //   if (!client) {
  //     client = useContext().client;
  //   }

  //   return tanstack.createInfiniteQuery({
  //     queryKey: keyAndInput,
  //     queryFn: () => {
  //       throw new Error("TODO"); // TODO: Finish this
  //     },
  //     ...(rawOpts as any),
  //   });
  // }

  return {
    _rspc_def: undefined! as P, // This allows inferring the operations type from TS helpers
    Provider: (props: Solid.ParentProps<rspc.Context<P>>) => (
      <Context.Provider
        value={{
          client: props.client,
          queryClient: props.queryClient,
        }}
      >
        {props.children}
      </Context.Provider>
    ),
    useContext: helpers.useContext,
    createQuery,
    // createInfiniteQuery,
    createMutation,
    createSubscription,
  };
}
