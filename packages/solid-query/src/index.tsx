import * as rspc from "@rspc/client";
import { ProceduresDef } from "@rspc/client";
import * as tanstack from "@tanstack/solid-query";
import {
  // CreateInfiniteQueryOptions,
  // CreateInfiniteQueryResult,
  CreateMutationOptions,
  CreateQueryOptions,
  QueryClientProvider,
} from "@tanstack/solid-query";
import {
  ParentProps,
  createContext,
  createEffect,
  on,
  onCleanup,
} from "solid-js";
import * as Solid from "solid-js";
import {
  SubscriptionOptions,
  BaseOptions,
  Context,
  handleSubscription,
  throwOnError,
  createQueryHookHelpers,
} from "@rspc/query-core";

export function createSolidQueryHooks<P extends ProceduresDef>() {
  type TBaseOptions = BaseOptions<P>;

  const Context = createContext<Context<P> | null>(null);

  const helpers = createQueryHookHelpers({
    useContext: () => Solid.useContext(Context),
  });

  function createQuery<K extends P["queries"]["key"] & string>(
    keyAndInput: () => [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: Omit<
      CreateQueryOptions<
        rspc.inferQueryResult<P, K>,
        rspc.inferQueryError<P, K>,
        rspc.inferQueryResult<P, K>,
        () => [K, rspc.inferQueryInput<P, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ) {
    const [client, rawOpts] = helpers.useExtractOps(opts ?? {});

    return tanstack.createQuery({
      queryKey: () => helpers.mapQueryKey(keyAndInput() as any, client),
      queryFn: () => client.query(keyAndInput()).then(throwOnError),
      ...rawOpts,
    });
  }

  function createMutation<
    K extends P["mutations"]["key"] & string,
    TContext = unknown
  >(
    key: K | [K],
    opts?: CreateMutationOptions<
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

    return tanstack.createMutation({
      mutationFn: async (input) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client.mutation([actualKey, input] as any).then(throwOnError);
      },
      ...rawOpts,
    });
  }

  function createSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >(
    keyAndInput: () => [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ],
    opts: () => SubscriptionOptions<rspc.inferSubscription<P, K>> & TBaseOptions
  ) {
    return createEffect(
      on(
        () => [keyAndInput(), opts?.()] as const,
        ([keyAndInput, opts]) => {
          const [client, rawOpts] = helpers.useExtractOps(opts ?? {});

          // TODO: solid-start useRequest

          const cleanup = handleSubscription<P, K>({
            client,
            keyAndInput,
            opts: rawOpts,
          });

          onCleanup(() => cleanup?.());
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
    Provider: (props: ParentProps<Context<P>>) => (
      <Context.Provider
        value={{
          client: props.client,
          queryClient: props.queryClient,
        }}
      >
        <QueryClientProvider client={props.queryClient}>
          {props.children}
        </QueryClientProvider>
      </Context.Provider>
    ),
    useContext: helpers.useContext,
    createQuery,
    // createInfiniteQuery,
    createMutation,
    createSubscription,
  };
}
