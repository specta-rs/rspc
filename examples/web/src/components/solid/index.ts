import { createClient } from "@rspc/client";
import { createSolidQueryProxy } from "@rspc/solid-query";
import { QueryClient } from "@tanstack/solid-query";

import type { Procedures } from "../../../../axum/bindings";

export const client = createClient<Procedures>();
export const rspc = createSolidQueryProxy<Procedures>();
export const queryClient = new QueryClient();

export * from "./Provider";
export * from "./Component";
