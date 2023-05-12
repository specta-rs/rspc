import { assert, test } from "vitest";
import { createRspcRoot, createRspcClient, httpLink } from "@rspc/client/v2";
import { Procedures } from "../../../examples/bindings";
import { createReactQueryHooks, createReactQueryRoot } from "./v2";

test("plain client and hooks", async () => {
  const rspcClient = createRspcClient<Procedures>({
    links: [httpLink({ url: "http://localhost:4000/rspc" })],
  });

  const plainHooks = createReactQueryHooks(rspcClient);

  assert(plainHooks.Provider);
  assert(plainHooks.useContext);
  assert(plainHooks.useQuery);
  assert(plainHooks.useInfiniteQuery);
  assert(plainHooks.useMutation);
  assert(plainHooks.useSubscription);
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

  const rspcClientBuilder = rspcRoot
    .createClientBuilder()
    .addSubClient(
      "library",
      rspcRoot.dangerously_createClientBuilder<ErrorProcedures>({
        dangerously_mapQueryKey: (keyAndInput) => keyAndInput,
      })
    )
    .addSubClient(
      "bruh",
      rspcRoot.dangerously_createClientBuilder<ErrorProcedures>({
        dangerously_mapQueryKey: (keyAndInput) => keyAndInput,
      })
    );

  const rspcClient = rspcClientBuilder.build({
    links: [httpLink({ url: "http://localhost:4000/rspc" })],
  });

  const rspcReact = createReactQueryRoot({
    builder: rspcClientBuilder,
  });

  const rspcHooks = rspcReact.createHooks();

  const {
    useQuery: useLibraryQuery,
    useMutation: useLibraryMutation,
    useSubscription: useLibrarySubscription,
  } = rspcReact.createHooks({ subClient: "library" });
});
