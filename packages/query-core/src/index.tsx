import { AlphaClient, ProceduresDef, ProcedureDef } from "@rspc/client";
import * as rspc from "@rspc/client";
import {
  MutationObserverOptions,
  QueryClient,
  QueryObserverOptions,
} from "@tanstack/query-core";

export * from "@rspc/client";

// TODO: Reuse one from client but don't export it in public API
export type KeyAndInput = [string] | [string, any];

export interface BaseOptions<TProcedures extends ProceduresDef> {
  rspc?: {
    client?: AlphaClient<TProcedures>;
  };
}

export interface SubscriptionOptions<P extends ProcedureDef> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: P["result"]) => void;
  onError?: (err: P["error"] | rspc.Error) => void;
}

export interface Context<TProcedures extends ProceduresDef> {
  client: AlphaClient<TProcedures>;
  queryClient: QueryClient;
}

export type QueryOptionsOmit<T> = Omit<T, "queryKey" | "queryFn">;
export type HookOptions<P extends ProceduresDef, T> = T & BaseOptions<P>;

export function throwOnError<TOk, TError>(res: rspc.Result<TOk, TError>) {
  if (res.status === "ok") return res.data;
  else return Promise.reject(res.error);
}

export function createQueryHookHelpers<P extends ProceduresDef>(args: {
  useContext(): Context<P> | null;
}) {
  type TBaseOptions = BaseOptions<P>;

  function useContext() {
    const ctx = args.useContext();
    if (!ctx) throw new Error("rspc context provider not found!");
    return ctx;
  }

  function useClient() {
    return useContext().client;
  }

  function useExtractOps<T extends TBaseOptions>(opts: T) {
    const { rspc, ...rawOpts } = opts;

    return [rspc?.client ?? useClient(), rawOpts] as const;
  }

  function mapQueryKey(keyAndInput: KeyAndInput, client: AlphaClient<P>) {
    return (client as any).mapQueryKey?.(keyAndInput) || keyAndInput;
  }

  type MaybeFn<T> = T | (() => T);

  function getMaybeFnValue<T extends MaybeFn<any>>(f: T) {
    if (typeof f === "function") return f();
    else return f;
  }

  function useQueryArgs<
    K extends rspc.inferQueries<P>["key"] & string,
    O extends QueryOptionsOmit<
      QueryObserverOptions<
        rspc.inferQueryResult<P, K>,
        rspc.inferQueryError<P, K> | rspc.Error,
        rspc.inferQueryResult<P, K>,
        rspc.inferQueryResult<P, K>,
        [K, rspc.inferQueryInput<P, K>]
      >
    > &
      TBaseOptions
  >(
    keyAndInput: MaybeFn<
      [key: K, ...input: rspc._inferProcedureHandlerInput<P, "queries", K>]
    >,
    opts?: O
  ) {
    const [client, rawOpts] = useExtractOps(opts ?? {});

    return {
      queryKey:
        typeof keyAndInput === "function"
          ? () => mapQueryKey(keyAndInput() as any, client)
          : mapQueryKey(keyAndInput as any, client),
      queryFn: () =>
        client.query(getMaybeFnValue(keyAndInput)).then(throwOnError),
      ...rawOpts,
    };
  }

  function useMutationArgs<
    K extends rspc.inferMutations<P>["key"] & string,
    O extends Omit<
      MutationObserverOptions<
        rspc.inferMutationResult<P, K>,
        rspc.inferMutationError<P, K> | rspc.Error,
        rspc.inferMutationInput<P, K> extends never
          ? undefined
          : rspc.inferMutationInput<P, K>,
        any
      >,
      "_defaulted" | "variables"
    > &
      TBaseOptions
  >(key: K | [K], opts?: O) {
    const [client, rawOpts] = useExtractOps(opts ?? {});

    return {
      mutationFn: async (input: any) => {
        const actualKey = Array.isArray(key) ? key[0] : key;
        return client.mutation([actualKey, input] as any).then(throwOnError);
      },
      ...rawOpts,
    };
  }

  function handleSubscription<
    K extends rspc.inferSubscriptions<P>["key"] & string
  >({
    client,
    keyAndInput,
    opts,
  }: {
    client: AlphaClient<P>;
    keyAndInput: [
      key: K,
      ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K>
    ];
    opts: Pick<
      HookOptions<P, SubscriptionOptions<rspc.inferSubscription<P, K>>>,
      "enabled" | "onData" | "onError"
    >;
  }) {
    if (!(opts.enabled ?? true)) return;

    let unsubscribed = false;
    const isStopped = () => unsubscribed;

    const unsubscribe = client.addSubscription(keyAndInput, {
      onData(data) {
        if (!isStopped()) opts.onData(data);
      },
      onError(error) {
        if (!isStopped()) opts.onError?.(error);
      },
    });

    return () => {
      unsubscribed = true;
      unsubscribe();
    };
  }

  return {
    useContext,
    useClient,
    useExtractOps,
    mapQueryKey,
    useQueryArgs,
    useMutationArgs,
    handleSubscription,
  };
}
