import * as rspc from "@rspc/client";
import { AlphaClient, ProcedureDef, ProceduresDef } from "@rspc/client";
import * as tanstack from "@tanstack/svelte-query";
import { CreateMutationOptions, CreateQueryOptions } from "@tanstack/svelte-query";
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
export declare function createSvelteQueryHooks<P extends ProceduresDef>(): {
    _rspc_def: P;
    useContext: () => {
        client: rspc.AlphaClient<P>;
        queryClient: tanstack.QueryClient;
    };
    createQuery: <K extends P["queries"]["key"] & string>(keyAndInput: [key: K, ...input: rspc._inferProcedureHandlerInput<P, "queries", K>], opts?: (Omit<tanstack.CreateQueryOptions<rspc.inferQueryResult<P, K>, rspc.inferQueryError<P, K>, rspc.inferQueryResult<P, K>, [K, rspc.inferQueryInput<P, K>]>, "queryKey" | "queryFn"> & BaseOptions<P>) | undefined) => tanstack.CreateQueryResult<rspc.inferQueryResult<P, K>, rspc.Error | Extract<rspc.inferProcedures<rspc.inferProcedures<rspc.inferProcedures<P>>>["queries"], {
        key: K;
    }>["error"]>;
    createMutation: <K_1 extends P["mutations"]["key"] & string, TContext = unknown>(key: K_1 | [K_1], opts?: (tanstack.CreateMutationOptions<rspc.inferMutationResult<P, K_1>, rspc.inferMutationError<P, K_1>, rspc.inferMutationInput<P, K_1> extends never ? undefined : rspc.inferMutationInput<P, K_1>, TContext> & BaseOptions<P>) | undefined) => tanstack.CreateMutationResult<rspc.inferMutationResult<P, K_1>, rspc.Error | Extract<rspc.inferProcedures<rspc.inferProcedures<rspc.inferProcedures<P>>>["mutations"], {
        key: K_1;
    }>["error"], rspc.inferMutationInput<P, K_1> extends never ? undefined : rspc.inferMutationInput<P, K_1>, TContext>;
    createSubscription: <K_2 extends rspc.inferSubscriptions<P>["key"] & string>(keyAndInput: [key: K_2, ...input: rspc._inferProcedureHandlerInput<P, "subscriptions", K_2>], opts?: (SubscriptionOptions<Extract<rspc.inferProcedures<rspc.inferProcedures<P>>["subscriptions"], {
        key: K_2;
    }>> & BaseOptions<P>) | undefined) => void;
};
//# sourceMappingURL=index.d.ts.map