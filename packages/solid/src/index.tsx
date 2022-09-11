import { createContext, useContext } from "solid-js";
import {
  Client,
  inferMutationResult,
  inferProcedureKey,
  inferQueryResult,
  Procedures,
  RSPCError,
} from "@rspc/client";
import {
  QueryClient,
  QueryObserverOptions,
  QueryObserverResult,
  MutationObserverResult,
  MutationObserverOptions,
  MutateFunction,
} from "@tanstack/query-core";
import {
  createQuery as _createQuery,
  createMutation as _createMutation,
} from "@adeora/solid-query";
import { inferMutationInput } from "@rspc/client";

export { QueryClient } from "@tanstack/query-core";

declare type CreateMutateFunction<
  TData = unknown,
  TError = unknown,
  TVariables = void,
  TContext = unknown
> = (
  ...args: Parameters<MutateFunction<TData, TError, TVariables, TContext>>
) => void;

declare type CreateMutateAsyncFunction<
  TData = unknown,
  TError = unknown,
  TVariables = void,
  TContext = unknown
> = MutateFunction<TData, TError, TVariables, TContext>;

declare type Override<A, B> = {
  [K in keyof A]: K extends keyof B ? B[K] : A[K];
};

interface Context<T extends Procedures> {
  client: Client<T>;
  queryClient: QueryClient;
}

export function createSolidQueryHooks<Operations extends Procedures>() {
  const rspcCtx = createContext<Context<Operations>>(undefined!);
  const ReactQueryContext = createContext<QueryClient>(undefined!);

  function useRspcContext() {
    const ctx = useContext(rspcCtx);
    if (ctx?.queryClient === undefined) throw new Error("The rspc "); // TODO: Error msg
    return ctx;
  }

  function createQuery<K extends Operations["queries"]["key"][0]>(
    key: inferProcedureKey<Operations, "queries", K>,
    options?: Omit<
      QueryObserverOptions<inferQueryResult<Operations, K>, RSPCError>,
      "queryKey" | "queryFn" | "initialData"
    > & {
      initialData?: () => undefined;
    } & {
      rspc?: {
        client?: Client<Operations>;
      };
    }
  ): QueryObserverResult<inferQueryResult<Operations, K>, RSPCError> {
    const ctx = useRspcContext();
    // @ts-ignore // TODO
    return _createQuery(
      () => key,
      async () => (options?.rspc?.client || ctx.client).query(key), // TODO: Put this feature of providing custom client into React version
      // @ts-ignore // TODO
      {
        ...options,
        context: ReactQueryContext,
      }
    );
  }

  function createMutation<K extends Operations["mutations"]["key"][0]>(
    key: K,
    options?: Omit<
      MutationObserverOptions<
        inferMutationResult<Operations, K>,
        RSPCError,
        inferMutationInput<Operations, K>
      >,
      "mutationKey" | "mutationFn" | "_defaulted" | "variables"
    > & {
      initialData?: () => undefined;
    } & {
      rspc?: {
        client?: Client<Operations>;
      };
    }
  ): Override<
    MutationObserverResult<
      inferMutationResult<Operations, K>,
      RSPCError,
      inferMutationInput<Operations, K>
    >,
    {
      mutate: CreateMutateFunction<
        inferMutationResult<Operations, K>,
        RSPCError,
        inferMutationInput<Operations, K>
      >;
    }
  > & {
    mutateAsync: CreateMutateAsyncFunction<
      inferMutationResult<Operations, K>,
      RSPCError,
      inferMutationInput<Operations, K>
    >;
  } {
    const ctx = useRspcContext();
    return _createMutation(
      async (data) =>
        (options?.rspc?.client || ctx.client).mutation([key, data]),
      {
        ...options,
        context: ReactQueryContext,
      }
    );
  }

  return {
    _rspc_def: undefined as Operations, // This allows inferring the operations type from TS helpers
    Provider: (props: {
      children?: any; // TODO: JSX.Element;
      client: Client<Operations>;
      queryClient: QueryClient;
    }) => {
      return (
        <rspcCtx.Provider
          value={{
            client: props.client,
            queryClient: props.queryClient,
          }}
        >
          <ReactQueryContext.Provider value={props.queryClient}>
            {props.children}
          </ReactQueryContext.Provider>
        </rspcCtx.Provider>
      );
    },
    ctx: rspcCtx,
    ReactQueryContext,
    createQuery,
    createMutation,
    // useSubscription, // TODO
    // useDehydratedState, // TODO
    // useInfiniteQuery, // TODO
  };
}
