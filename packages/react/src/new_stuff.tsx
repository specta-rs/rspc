import { ProceduresDef } from "@rspc/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createContext, PropsWithChildren } from "react";

interface ProviderArgs {
  queryClient: QueryClient;
  client: any;
}

interface Context<P extends ProceduresDef> {
  queryClient: QueryClient;
  client: any & P;
}

interface LibraryArgs<T> {
  library_id: string;
  arg: T;
}

type Procedures = {
  queries:
    | {
        key: "a";
        input: string;
        result: number;
      }
    | {
        key: "b";
        input: LibraryArgs<string>;
        result: string;
      };
  mutations: never;
  subscriptions: never;
};

export function createReactRoot<P extends ProceduresDef>() {
  const context = createContext<Context<P>>(undefined!);

  const Provider = ({
    children,
    client,
    queryClient,
  }: PropsWithChildren<ProviderArgs>) => (
    <context.Provider
      value={{
        client,
        queryClient,
      }}
    >
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </context.Provider>
  );

  return {
    Provider,
  };
}
