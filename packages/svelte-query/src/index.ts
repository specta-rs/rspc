import { onDestroy } from "svelte";
import * as tanstack from "@tanstack/svelte-query";
import * as rspc from "@rspc/query-core";
import { getRspcClientContext } from "./context";
import { Readable, derived, readable } from "svelte/store";

function isSvelteStore<T extends object>(
  obj: tanstack.StoreOrVal<T>
): obj is Readable<T> {
  return "subscribe" in obj && typeof obj.subscribe === "function";
}

function enforceSvelteStore<T extends object>(
  obj: tanstack.StoreOrVal<T>
): Readable<T> {
  if (isSvelteStore(obj)) {
    return obj;
  }
  return readable(obj);
}

export function createRSPCSolidQuery<P extends rspc.ProceduresDef>({
  client,
}: {
  client: rspc.Client<P>;
}) {
  return createRawRSPCSvelteQuery({ root: client._root });
}

export function createRawRSPCSvelteQuery<P extends rspc.ProceduresDef>(_: {
  root: rspc.Root<P>;
}) {
  const helpers = rspc.createQueryHookHelpers({
    useContext: getRspcClientContext<P>,
  });

  type CreateQueryOptions<K extends rspc.inferQueries<P>["key"] & string> =
    rspc.HookOptions<
      P,
      rspc.QueryOptionsOmit<
        tanstack.CreateQueryOptions<
          rspc.inferQueryResult<P, K>,
          rspc.inferQueryError<P, K>,
          rspc.inferQueryResult<P, K>,
          [K, rspc.inferQueryInput<P, K>]
        >
      >
    >;

  function createQuery<K extends rspc.inferQueries<P>["key"] & string>(
    keyAndInput: tanstack.StoreOrVal<
      [key: K, ...input: rspc._inferProcedureHandlerInput<P, "queries", K>]
    >,
    opts?: tanstack.StoreOrVal<CreateQueryOptions<K>>
  ) {
    return tanstack.createQuery(
      derived(
        [
          enforceSvelteStore(keyAndInput),
          enforceSvelteStore<CreateQueryOptions<K>>(opts ?? {}),
        ],
        ([$keyAndInput, $opts]) => helpers.useQueryArgs($keyAndInput, $opts)
      )
    );
  }

  type CreateMutationOptions<
    K extends rspc.inferMutations<P>["key"] & string,
    TContext = unknown
  > = rspc.HookOptions<
    P,
    tanstack.CreateMutationOptions<
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
    return tanstack.createMutation(helpers.useMutationArgs(key, opts));
  }

  function createSubscription<
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

    const cleanup = helpers.handleSubscription({
      client,
      keyAndInput,
      opts: rawOpts,
    });

    return onDestroy(() => cleanup?.());
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
    useContext: helpers.useContext,
    createQuery,
    // createInfiniteQuery,
    createMutation,
    createSubscription,
  };
}
