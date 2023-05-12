import { test, assert } from "vitest";
import { createRspcClient, createRspcRoot } from "./client";
import { Procedures } from "../../../../examples/bindings";
import { httpLink } from "./links/httpLink";

test("plain client", async () => {
  const client = createRspcClient<Procedures>({
    links: [httpLink({ url: "http://localhost:4000/rspc" })],
  });

  assert(client.query);
  assert(client.mutation);
  assert(client.addSubscription);
});

test("dangerously_mapQueryKey", async () => {
  type ErrorProcedure<T extends keyof Procedures> = Exclude<
    Extract<Procedures[T], { input: string }>,
    { input: never }
  >;

  type ErrorProcedures = {
    queries: ErrorProcedure<"queries">;
    mutations: ErrorProcedure<"mutations">;
    subscriptions: ErrorProcedure<"subscriptions">;
  };

  const rspcRoot = createRspcRoot<Procedures>();

  const rspcClientBuilder = rspcRoot.createClientBuilder().addSubClient(
    "error",
    rspcRoot.dangerously_createClientBuilder<ErrorProcedures>({
      dangerously_mapQueryKey: (keyAndInput) => keyAndInput,
    })
  );

  const client = rspcClientBuilder.build({
    links: [httpLink({ url: "http://localhost:4000/rspc" })],
  });

  assert(client.subClient("error"));
});
