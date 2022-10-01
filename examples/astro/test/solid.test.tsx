/** @jsxImportSource solid-js */
import { RSPCError, Client, createClient, NoOpTransport } from "@rspc/client";
import { createSolidQueryHooks } from "@rspc/solid";
import { QueryClient } from "@tanstack/solid-query";
import { Procedures } from "./bindings";

export const rspc = createSolidQueryHooks<Procedures>();

function assert<T>(t: T) {}

// createContext
assert<Client<Procedures>>(rspc.useContext().client);

// createQuery
const { data, error } = rspc.createQuery(() => ["noArgQuery"], {
  onSuccess(data) {
    assert<string>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
});
assert<string | undefined>(data);
assert<RSPCError | null>(error);

const { data: data2, error: error2 } = rspc.createQuery(
  () => ["singleArgQuery", 42],
  {
    onSuccess(data) {
      assert<number>(data);
    },
    onError(err) {
      assert<RSPCError>(err);
    },
  }
);
assert<number | undefined>(data2);
assert<RSPCError | null>(error2);

// createInfiniteQuery
// TODO
// rspc.createInfiniteQuery(["paginatedQueryOnlyCursor"]);
// rspc.createInfiniteQuery(["paginatedQueryCursorAndArg", { my_param: 42 }]);

// createMutation
const {
  mutate,
  error: error3,
  data: data3,
} = rspc.createMutation("noArgMutation", {
  onSuccess(data) {
    assert<string>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
});
mutate(undefined);
assert<RSPCError | null>(error3);
assert<string | undefined>(data3);

const {
  mutate: mutate2,
  error: error4,
  data: data4,
} = rspc.createMutation("singleArgMutation", {
  onSuccess(data) {
    assert<number>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
});
mutate2(42);
assert<RSPCError | null>(error4);
assert<number | undefined>(data4);

// createSubscription
// rspc.createSubscription(() => ["noArgSubscription"], {
//   onStarted() {},
//   onData(data) {
//     assert<string>(data);
//   },
//   onError(err) {
//     assert<RSPCError>(err);
//   },
//   enabled: false,
// });

// rspc.createSubscription(() => ["singleArgSubscription", true], {
//   onStarted() {},
//   onData(data) {
//     assert<boolean>(data);
//   },
//   onError(err) {
//     assert<RSPCError>(err);
//   },
//   enabled: false,
// });

// Provider
const queryClient = new QueryClient();
const client = createClient<Procedures>({
  transport: new NoOpTransport(),
});

function NoChildrenWithProvider() {
  return (
    <div>
      <rspc.Provider client={client} queryClient={queryClient} />
    </div>
  );
}

function ChildrenWithProvider() {
  return (
    <div>
      <rspc.Provider client={client} queryClient={queryClient}>
        <h1>My App</h1>
      </rspc.Provider>
    </div>
  );
}
