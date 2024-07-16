import { createClient } from "@rspc/client";
import { render } from "@solidjs/testing-library";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { test } from "vitest";
import { createSolidQueryProxy } from ".";

type NestedProcedures = {
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

const rspc = createSolidQueryProxy<NestedProcedures>();

test("proxy", () => {
	const queryClient = new QueryClient();

	function Component() {
		const query = rspc.nested.procedures.one.createQuery(() => "test");

		const mutation = rspc.nested.procedures.two.createMutation();
		mutation.mutate("bruh");

		rspc.nested.procedures.three.createSubscription(
			() => "value",
			() => ({
				onData: (d) => {},
			}),
		);

		return null;
	}

	const client = createClient<NestedProcedures>();

	render(() => (
		<rspc.Provider client={client} queryClient={queryClient}>
			<QueryClientProvider client={queryClient}>
				<Component />
			</QueryClientProvider>
		</rspc.Provider>
	));
});
