import { createClient } from "@rspc/client";
import { QueryClient } from "@tanstack/svelte-query";
import { createSvelteQueryProxy } from "../src";

export type NestedProcedures = {
	nested: {
		procedures: {
			one: {
				kind: "query";
				input: string;
				result: number;
				error: boolean;
			};
			two: {
				kind: "mutation";
				input: string;
				result: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
			three: {
				kind: "subscription";
				input: string;
				result: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
		};
	};
};

export const queryClient = new QueryClient();
export const client = createClient<NestedProcedures>();
export const rspc = createSvelteQueryProxy<NestedProcedures>();
