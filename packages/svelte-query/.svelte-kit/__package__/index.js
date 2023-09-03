import * as tanstack from "@tanstack/svelte-query";
import { onDestroy } from "svelte";
import { getRspcClientContext } from "./context";
export function createSvelteQueryHooks() {
    const mapQueryKey = (keyAndInput, client) => client.mapQueryKey?.(keyAndInput) || keyAndInput;
    function useContext() {
        return getRspcClientContext();
    }
    function createQuery(keyAndInput, opts) {
        const { rspc, ...rawOpts } = opts ?? {};
        const client = opts?.rspc?.client ?? useContext().client;
        return tanstack.createQuery({
            queryKey: mapQueryKey(keyAndInput, client),
            queryFn: () => client.query(keyAndInput).then((res) => {
                if (res.status === "ok")
                    return res.data;
                else
                    return Promise.reject(res.error);
            }),
            ...rawOpts,
        });
    }
    function createMutation(key, opts) {
        const { rspc, ...rawOpts } = opts ?? {};
        const client = opts?.rspc?.client ?? useContext().client;
        return tanstack.createMutation({
            mutationFn: async (input) => {
                const actualKey = Array.isArray(key) ? key[0] : key;
                return client.mutation([actualKey, input]).then((res) => {
                    if (res.status === "ok")
                        return res.data;
                    else
                        throw res.error;
                });
            },
            ...rawOpts,
        });
    }
    function createSubscription(keyAndInput, opts) {
        const client = opts?.rspc?.client ?? useContext().client;
        if (!(opts?.enabled ?? true))
            return;
        let isStopped = false;
        const unsubscribe = client.addSubscription(keyAndInput, {
            onData: (data) => {
                if (!isStopped)
                    opts?.onData(data);
            },
            onError: (err) => {
                if (!isStopped)
                    opts?.onError?.(err);
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
        _rspc_def: undefined,
        useContext,
        createQuery,
        // createInfiniteQuery,
        createMutation,
        createSubscription,
    };
}
