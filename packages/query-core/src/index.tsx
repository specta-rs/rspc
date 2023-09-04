import { AlphaClient, ProceduresDef, ProcedureDef } from "@rspc/client";
import * as rspc from "@rspc/client";
import { QueryClient } from "@tanstack/query-core";

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

export function handleSubscription<
  P extends ProceduresDef,
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
    SubscriptionOptions<rspc.inferSubscription<P, K>> & BaseOptions<P>,
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

export function throwOnError<TOk, TError>(res: rspc.Result<TOk, TError>) {
  if (res.status === "ok") return res.data;
  else return Promise.reject(res.error);
}

export function createQueryHookHelpers<P extends ProceduresDef>(args: {
  useContext(): Context<P> | null;
}) {
  function useContext() {
    const ctx = args.useContext();
    if (!ctx) throw new Error("rspc context provider not found!");
    return ctx;
  }

  function useClient() {
    return useContext().client;
  }

  function useExtractOps<T extends BaseOptions<P>>(opts: T) {
    const { rspc, ...rawOpts } = opts;

    return [rspc?.client ?? useClient(), rawOpts] as const;
  }

  return {
    useContext,
    useClient,
    useExtractOps,
    mapQueryKey(keyAndInput: KeyAndInput, client: AlphaClient<P>) {
      return (client as any).mapQueryKey?.(keyAndInput) || keyAndInput;
    },
  };
}
