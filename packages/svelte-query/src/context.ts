import { AlphaClient, ProceduresDef } from "@rspc/client";
import { QueryClient } from "@tanstack/svelte-query";
import { getContext, setContext } from "svelte";

const _contextKey = "$$_rspcClient";

type Context<P extends ProceduresDef> = {
  client: AlphaClient<P>;
  queryClient: QueryClient;
};

/** Retrieves a Client from Svelte's context */
export const getRspcClientContext = <P extends ProceduresDef>(): Context<P> => {
  const ctx = getContext(_contextKey);

  if (!ctx) {
    throw new Error(
      "No rspc Client was found in Svelte context. Did you forget to wrap your component with RspcProvider?"
    );
  }

  return ctx as Context<P>;
};

/** Sets a Client on Svelte's context */
export const setRspcClientContext = (client: AlphaClient<any>): void => {
  setContext(_contextKey, client);
};
