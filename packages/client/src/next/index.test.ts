import { test } from "vitest";
import { createClient, observable } from ".";

type NestedProcedures = {
	nested: {
		procedures: {
			one: {
				kind: "query";
				input: string;
				output: number;
				error: boolean;
			};
			two: {
				kind: "mutation";
				input: string;
				output: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
			three: {
				kind: "subscription";
				input: string;
				output: { id: string; name: string };
				error: { status: "NOT_FOUND" };
			};
		};
	};
};

const client = createClient<NestedProcedures>(() => observable(() => {}));

test("proxy", () => {
	client.nested.procedures.one.query("test");
	client.nested.procedures.two.mutate("test");
	client.nested.procedures.three.subscribe("test");
});
