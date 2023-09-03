import * as rspc from "@rspc/client";
import { AlphaClient, ProcedureDef, ProceduresDef } from "@rspc/client";
import * as tanstack from "@tanstack/svelte-query";
import {
  // CreateInfiniteQueryOptions,
  // CreateInfiniteQueryResult,
  CreateMutationOptions,
  CreateMutationResult,
  CreateQueryOptions,
  CreateQueryResult,
  QueryClient,
  QueryClientProvider,
  hashQueryKey,
} from "@tanstack/svelte-query";
import { onDestroy, setContext } from "svelte";
import { getRspcClientContext } from "./context";

export interface BaseOptions<P extends ProceduresDef> {
  rspc?: {
    client?: AlphaClient<P>;
  };
}

export interface SubscriptionOptions<P extends ProcedureDef> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: P["result"]) => void;
  onError?: (err: P["error"] | rspc.Error) => void;
}

type KeyAndInput = [string] | [string, any];

export function createSvelteQueryHooks<P extends ProceduresDef>() {
  type TBaseOptions = BaseOptions<P>;

  const mapQueryKey: (
    keyAndInput: KeyAndInput,
    client: AlphaClient<P>
  ) => KeyAndInput = (keyAndInput, client) =>
    (client as any).mapQueryKey?.(keyAndInput) || keyAndInput;

  function useContext() {
    return getRspcClientContext<P>();
  }

  function createQuery<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: Omit<
      CreateQueryOptions<
        rspc.inferQueryResult<P, K>,
        rspc.inferQueryError<P, K>,
        rspc.inferQueryResult<P, K>,
        [K, rspc.inferQueryInput<P, K>]
      >,
      "queryKey" | "queryFn"
    > &
      TBaseOptions
  ) {
    const { rspc, ...rawOpts } = opts ?? {};

    const client = opts?.rspc?.client ?? useContext().client;

    return tanstack.createQuery({
      queryKey: mapQueryKey(keyAndInput as any, client) as any,
      queryFn: () =>
        client.query(keyAndInput).then((res) => {
          if (res.status === "ok") return res.data;
          else return Promise.reject(res.error);
        }),
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
    const { rspc, ...rawOpts } = opts ?? {};

    const client = opts?.rspc?.client ?? useContext().client;

    return tanstack.createMutation({
      mutationFn: async (input) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client.mutation([actualKey, input] as any).then((res) => {
          if (res.status === "ok") return res.data;
          else throw res.error;
        });
      },
      ...rawOpts,
    });
  }

  function createSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >(
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ],
    opts?: SubscriptionOptions<rspc.inferSubscription<P, K>> & TBaseOptions
  ) {
    const client = opts?.rspc?.client ?? useContext().client;

    if (!(opts?.enabled ?? true)) return;

    let isStopped = false;

    const unsubscribe = client.addSubscription(keyAndInput, {
      onData: (data) => {
        if (!isStopped) opts?.onData(data);
      },
      onError: (err) => {
        if (!isStopped) opts?.onError?.(err);
      },
    });

    return onDestroy(() => {
      isStopped = true;
      unsubscribe();
    });
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
    useContext,
    createQuery,
    // createInfiniteQuery,
    createMutation,
    createSubscription,
  };
}
