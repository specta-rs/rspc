import { Client, httpLink, ProceduresDef, Root } from "@rspc/client";
import React, { PropsWithChildren } from "react";
import { QueryClient } from "@tanstack/react-query";
import { createRawRSPCReactQuery } from "./";
import { expect, test } from "vitest";

/// @sd/client

const root = new Root<ProceduresDef>();

const libraryRoot = root.createChild<ProceduresDef>({
  mapQueryKey: (key) => key,
});

const rspcHooks = createRawRSPCReactQuery({ root });
const rspcLibraryHooks = createRawRSPCReactQuery({ root: libraryRoot });

/// @sd/desktop

const client = new Client({
  root,
  links: [httpLink({ url: "http://localhost:4000/rspc" })],
});

const libraryClient = client.createChild({
  root: libraryRoot,
});

const queryClient = new QueryClient();

export function Providers({ children }: PropsWithChildren) {
  return (
    <rspcHooks.Provider client={client} queryClient={queryClient}>
      <rspcLibraryHooks.Provider
        client={libraryClient}
        queryClient={queryClient}
      >
        {children}
      </rspcLibraryHooks.Provider>
    </rspcHooks.Provider>
  );
}

test("client roots", () => {
  expect(libraryClient._root).toBe(libraryRoot);
  expect(libraryRoot.parent).toBe(root);
});
