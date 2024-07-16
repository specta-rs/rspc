import { test } from "vitest";
import { createClient } from ".";

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

const client = createClient<NestedProcedures>();

test("proxy", () => {
	client.nested.procedures.one.query("test");
	client.nested.procedures.two.mutate("test");
	client.nested.procedures.three.subscribe("test");
});
