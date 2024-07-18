import { test } from "vitest";
import { createClient } from ".";

type NestedProcedures = {
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

const client = createClient<NestedProcedures>();

test("proxy", () => {
	client.nested.procedures.one.query("test");
	client.nested.procedures.two.mutate("test");
	client.nested.procedures.three.subscribe("test");
});
