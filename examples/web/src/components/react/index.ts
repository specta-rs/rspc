import { createClient } from "@rspc/client";
import { createReactQueryProxy } from "@rspc/react-query";
import { QueryClient } from "@tanstack/react-query";

import type { Procedures } from "../../../../axum/bindings";

export const client = createClient<Procedures>();
export const rspc = createReactQueryProxy<Procedures>();
export const queryClient = new QueryClient();

export * from "./Provider";
export * from "./Component";
