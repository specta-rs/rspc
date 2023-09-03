import { AlphaClient, ProceduresDef } from "@rspc/client";
import { QueryClient } from "@tanstack/svelte-query";
type Context<P extends ProceduresDef> = {
    client: AlphaClient<P>;
    queryClient: QueryClient;
};
/** Retrieves a Client from Svelte's context */
export declare const getRspcClientContext: <P extends ProceduresDef>() => Context<P>;
/** Sets a Client on Svelte's context */
export declare const setRspcClientContext: (client: AlphaClient<any>) => void;
export {};
//# sourceMappingURL=context.d.ts.map