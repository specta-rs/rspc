import type { VoidIfInputNull, SubscriptionObserver } from "@rspc/client/next";
import {
  type Client,
  type Procedure,
  type Procedures,
  createProceduresProxy,
  traverseClient,
} from "@rspc/client/next";
import * as tanstack from "@tanstack/solid-query";
import { skipToken } from "@tanstack/solid-query";

// TODO: these should be overloaded to support passing options object when input is null
export type RspcQueryOptions<P extends Procedure> = {
  <TQueryFnData extends P["output"], TData = TQueryFnData>(
    input: VoidIfInputNull<P> | tanstack.SkipToken,
    options?: Omit<
      ReturnType<
        tanstack.UndefinedInitialDataOptions<
          TQueryFnData,
          P["error"],
          TData,
          any
        >
      >,
      "queryKey" | "queryFn" | "queryHash" | "queryHashFn"
    >,
  ): tanstack.UndefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>;
  <TQueryFnData extends P["output"], TData = TQueryFnData>(
    input: VoidIfInputNull<P> | tanstack.SkipToken,
    options?: Omit<
      ReturnType<
        tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>
      >,
      "queryKey" | "queryFn" | "queryHash" | "queryHashFn"
    >,
  ): tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>;
};

type QueryMethods<P extends Procedure> = {
  queryOptions: RspcQueryOptions<P>;

  queryKey: (
    input?: Partial<VoidIfInputNull<P>>,
  ) => tanstack.DataTag<ReadonlyArray<unknown>, P["output"], P["error"]>;
};

export type RspcMutationOptions<P extends Procedure> = <TCtx = unknown>(
  options?: Omit<
    ReturnType<
      tanstack.UseMutationOptions<P["output"], P["error"], P["input"], TCtx>
    >,
    "mutationKey" | "mutationFn"
  >,
) => tanstack.UseMutationOptions<P["output"], P["error"], P["input"], TCtx>;

type MutationMethods<P extends Procedure> = {
  mutationOptions: RspcMutationOptions<P>;

  mutationKey: () => ReadonlyArray<unknown>;
};

export type RspcSubscriptionOptions<TOut, TError> = SubscriptionObserver<TOut, TError> & {
  subscribe: (innerOpts: SubscriptionObserver<TOut, TError>) => void;
  enabled: boolean;
  queryKey: unknown;
}

type SubscriptionMethods<P extends Procedure> = {
  subscriptionOptions: (
    input: VoidIfInputNull<P> | tanstack.SkipToken,
    opts: SubscriptionObserver<P["output"], P["error"]> & Partial<{ enabled: boolean }>,
  ) => RspcSubscriptionOptions<P["output"], P["error"]>;
};

export type ProcedureProxyMethods<P extends Procedure> =
  P["kind"] extends "query"
  ? QueryMethods<P>
  : P["kind"] extends "mutation"
  ? MutationMethods<P>
  : P["kind"] extends "subscription"
  ? SubscriptionMethods<P>
  : never;

export type OptionsProceduresProxy<P extends Procedures> = {
  [K in keyof P]: P[K] extends Procedure
  ? ProcedureProxyMethods<P[K]>
  : P[K] extends Procedures
  ? OptionsProceduresProxy<P[K]>
  : never;
};

type UtilsMethods =
  | keyof QueryMethods<any>
  | keyof MutationMethods<any>
  | keyof SubscriptionMethods<any>;

export function createRSPCOptionsProxy<P extends Procedures>(
  client: Client<P>,
): OptionsProceduresProxy<P> {
  return createProceduresProxy<OptionsProceduresProxy<P>>(({ args, path }) => {
    const option = path.pop() as UtilsMethods;

    const methods: Record<UtilsMethods, () => unknown> = {
      queryOptions: () => {
        return () =>
          tanstack.queryOptions({
            ...(args[1] ?? {}),
            queryKey: [path, args[0]],
            queryFn:
              args[0] === tanstack.skipToken
                ? args[0]
                : () => (traverseClient(client, path) as any).query(args[0]),
          });
      },
      queryKey: () => {
        return [path, args[0]];
      },
      mutationOptions: () => {
        return () => ({
          ...(args[0] ?? {}),
          mutationKey: [path],
          mutationFn: (input: unknown) =>
            (traverseClient(client, path) as any).mutate(input),
        });
      },
      mutationKey: () => {
        return [path];
      },
      subscriptionOptions: () => {
        const opts = args[1]
        const enabled = "enabled" in opts ? !!opts.enabled : args[0] !== skipToken
        return {
          ...opts,
          queryKey: [path, args[0]],
          enabled,
          subscribe: (innerOpts: SubscriptionObserver<any, any>) =>
            (traverseClient(client, path) as any).subscribe(args[0], innerOpts),
        };
      },
    };

    if (option in methods) {
      return methods[option]();
    }

    throw new Error(`Invalid proxy method '${option}'`);
  });
}

