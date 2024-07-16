import { QueryClient } from "@tanstack/svelte-query";
import { createSvelteQueryProxy } from "../src";
import { createClient } from "@rspc/client";

export type NestedProcedures = {
	nested: {
		procedures: {
			one: {
				variant: "query";
				input: string;
				result: number;
				error: boolean;
			};
			two: {
				variant: "mutation";
				input: string;
				result: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
			three: {
				variant: "subscription";
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
