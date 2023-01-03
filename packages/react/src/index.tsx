import {
  createContext,
  ReactElement,
  useContext as _useContext,
  useEffect,
} from "react";
import {
  QueryClient,
  useQuery as __useQuery,
  useInfiniteQuery as __useInfiniteQuery,
  useMutation as __useMutation,
  UseQueryResult,
  UseQueryOptions,
  UseMutationResult,
  UseMutationOptions,
  hashQueryKey,
  QueryClientProvider,
} from "@tanstack/react-query";
import {
  Client as _Client,
  RSPCError,
  _inferProcedureHandlerInput,
  _inferInfiniteQueryProcedureHandlerInput,
  ProcedureDef,
  ProceduresDef,
  createVanillaClient as _createVanillaClient,
  ClientArgs,
  HookOptions,
} from "@rspc/client";

export interface BaseOptions<TProcedures extends ProceduresDef> {
  rspc?: {
    client?: _Client<TProcedures>;
    abortOnUnmount?: boolean;
  };
}

export interface SubscriptionOptions<TOutput> {
  enabled?: boolean;
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

export interface Context<
  TProcedures extends ProceduresDef,
  TQueries extends ProcedureDef,
  TMutations extends ProcedureDef,
  TSubscriptions extends ProcedureDef
> {
  client: _Client<TProcedures, TQueries, TMutations, TSubscriptions>;
  queryClient: QueryClient;
}

export function internal_createReactHooksFactory<
  TBaseProcedures extends ProceduresDef = never,
  TQueries extends ProcedureDef = TBaseProcedures["queries"],
  TMutations extends ProcedureDef = TBaseProcedures["mutations"],
  TSubscriptions extends ProcedureDef = TBaseProcedures["subscriptions"]
>() {
  const Context = createContext<
    Context<TBaseProcedures, TQueries, TMutations, TSubscriptions>
  >(undefined!);

  const Provider = ({
    children,
    client,
    queryClient,
  }: {
    children?: ReactElement;
    client: _Client<TBaseProcedures, TQueries, TMutations, TSubscriptions>;
    queryClient: QueryClient;
  }) => (
    <Context.Provider
      value={{
        client,
        queryClient,
      }}
    >
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </Context.Provider>
  );

  function createClient(opts: ClientArgs) {
    return _createVanillaClient<
      TBaseProcedures,
      TQueries,
      TMutations,
      TSubscriptions
    >(opts);
  }

  return {
    Provider,
    createClient,
    createHooks<
      TBaseProcedures extends ProceduresDef = never,
      TQueries extends ProcedureDef = TBaseProcedures["queries"],
      TMutations extends ProcedureDef = TBaseProcedures["mutations"],
      TSubscriptions extends ProcedureDef = TBaseProcedures["subscriptions"]
    >(hookOpts?: HookOptions) {
      type TProcedures = {
        queries: TQueries;
        mutations: TMutations;
        subscriptions: TSubscriptions;
      };
      type TBaseOptions = BaseOptions<TProcedures>;

      type TQuery<K extends TQueries["key"] & string> = Extract<
        TQueries,
        { key: K }
      >;
      type TMutation<K extends TMutations["key"] & string> = Extract<
        TMutations,
        { key: K }
      >;
      type TSubscription<K extends TSubscriptions["key"] & string> = Extract<
        TSubscriptions,
        { key: K }
      >;
      type CustomClient = _Client<
        TBaseProcedures,
        TQueries,
        TMutations,
        TSubscriptions
      >;

      const customHooks = hookOpts?.internal?.customHooks?.();

      function useContext() {
        const ctx = _useContext(Context);
        if (ctx?.queryClient === undefined)
          throw new Error(
            "The rspc context has not been set. Ensure you have the `<rspc.Provider>` component higher up in your component tree."
          );
        return ctx;
      }

      function useQuery<
        K extends TQueries["key"] & string,
        TQueryFnData extends TQuery<K>["result"],
        TData extends TQuery<K>["result"]
      >(
        keyAndInput: [
          key: K,
          ...input: TQuery<K>["input"] extends null ? [] : [TQuery<K>["input"]]
        ],
        opts?: Omit<
          UseQueryOptions<
            TQueryFnData,
            RSPCError,
            TData,
            [
              key: K,
              ...input: TQuery<K>["input"] extends null
                ? []
                : [TQuery<K>["input"]]
            ]
          >,
          "queryKey" | "queryFn"
        > &
          TBaseOptions
      ): UseQueryResult<TData, RSPCError> {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client as any; // TODO: Types
        }

        let newKeyAndInput =
          customHooks?.mapQueryKey?.(keyAndInput as any) || keyAndInput;

        const useQuery = customHooks?.dangerous?.useQuery ?? __useQuery;
        return useQuery(
          newKeyAndInput as any,
          async () => {
            const next = (keyAndInput: [string] | [string, any]) =>
              // TODO: Abort controller and stuff from ctx
              client!.query(keyAndInput as any);

            const data =
              customHooks?.doQuery?.(newKeyAndInput as any, next) ??
              next(newKeyAndInput as any);

            return await data;
          },
          rawOpts as any
        );
      }

      function useMutation<
        K extends TMutations["key"] & string,
        TContext = unknown
      >(
        key: K | [K],
        opts?: UseMutationOptions<
          TMutation<K>["result"],
          RSPCError,
          TMutation<K>["input"] extends null
            ? undefined
            : TMutation<K>["input"],
          TContext
        > &
          TBaseOptions
      ): UseMutationResult<
        TMutation<K>["result"],
        RSPCError,
        TMutation<K>["input"] extends null ? undefined : TMutation<K>["input"],
        TContext
      > {
        const { rspc, ...rawOpts } = opts ?? {};
        let client = rspc?.client;
        if (!client) {
          client = useContext().client as any; // TODO
        }

        const useMutation =
          customHooks?.dangerous?.useMutation ?? __useMutation;
        return useMutation(async (input: any) => {
          const keyAndInput = [Array.isArray(key) ? key[0] : key, input];

          const next = (keyAndInput: [string] | [string, any]) =>
            // TODO: Abort controller and stuff from ctx
            client!.mutation(keyAndInput as any);

          const data =
            customHooks?.doMutation?.(keyAndInput as any, next) ??
            next(keyAndInput as any);

          return await data;
        }, rawOpts as any);
      }

      function useSubscription<
        K extends TSubscriptions["key"] & string,
        TData = TSubscription<K>["result"]
      >(
        keyAndInput: [
          key: K,
          ...input: TSubscription<K>["input"] extends null
            ? []
            : [TSubscription<K>["input"]]
        ],
        opts: SubscriptionOptions<TData>
      ) {
        const enabled = opts?.enabled ?? true;
        const queryKey = hashQueryKey(keyAndInput);
        const { client } = useContext();

        return useEffect(() => {
          if (!enabled) {
            return;
          }
          let isStopped = false;
          const subscription = client.subscription<any>(keyAndInput as any, {
            onStarted: () => {
              if (!isStopped) {
                opts.onStarted?.();
              }
            },
            onData: (data) => {
              if (!isStopped) {
                opts.onData(data);
              }
            },
            onError: (err) => {
              if (!isStopped) {
                opts.onError?.(err);
              }
            },
          });
          return () => {
            isStopped = true;
            subscription.unsubscribe();
          };
        }, [queryKey, enabled]);
      }

      return {
        createClient,
        useContext,
        Provider,
        useQuery,
        // useInfiniteQuery, // TODO
        useMutation,
        useSubscription,
      };
    },
  };
}

export function createReactHooks<
  TBaseProcedures extends ProceduresDef = never,
  TQueries extends ProcedureDef = TBaseProcedures["queries"],
  TMutations extends ProcedureDef = TBaseProcedures["mutations"],
  TSubscriptions extends ProcedureDef = TBaseProcedures["subscriptions"]
>(hookOpts?: HookOptions) {
  let hooks = internal_createReactHooksFactory<
    TBaseProcedures,
    TQueries,
    TMutations,
    TSubscriptions
  >();
  return hooks.createHooks<
    TBaseProcedures,
    TQueries,
    TMutations,
    TSubscriptions
  >(hookOpts);
}
