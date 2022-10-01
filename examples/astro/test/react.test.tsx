import React from "react";
import { RSPCError, Client, createClient, NoOpTransport } from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { QueryClient } from "@tanstack/react-query";
import { Procedures } from "./bindings";

export const rspc = createReactQueryHooks<Procedures>();

function assert<T>(t: T) {}

// useContext
assert<Client<Procedures>>(rspc.useContext().client);

// useQuery
const { data, error } = rspc.useQuery(["noArgQuery"], {
  onSuccess(data) {
    assert<string>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
});
assert<string | undefined>(data);
assert<RSPCError | null>(error);

const { data: data2, error: error2 } = rspc.useQuery(["singleArgQuery", 42], {
  onSuccess(data) {
    assert<number>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
});
assert<number | undefined>(data2);
assert<RSPCError | null>(error2);

// useInfiniteQuery
// TODO
// rspc.useInfiniteQuery(["paginatedQueryOnlyCursor"]);
// rspc.useInfiniteQuery(["paginatedQueryCursorAndArg", { my_param: 42 }]);

// useMutation
const {
  mutate,
  error: error3,
  data: data3,
} = rspc.useMutation("noArgMutation", {
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
} = rspc.useMutation("singleArgMutation", {
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

// useSubscription
rspc.useSubscription(["noArgSubscription"], {
  onStarted() {},
  onData(data) {
    assert<string>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
  enabled: false,
});

rspc.useSubscription(["singleArgSubscription", true], {
  onStarted() {},
  onData(data) {
    assert<boolean>(data);
  },
  onError(err) {
    assert<RSPCError>(err);
  },
  enabled: false,
});

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
