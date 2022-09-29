import {
  inferProcedures,
  inferMutationInput,
  inferMutationResult,
  inferQueryInput,
  inferQueryResult,
  inferSubscriptionInput,
  inferSubscriptionResult,
  NoOpTransport,
  createClient,
  inferInfiniteQueries,
  inferInfiniteQueryResult,
  inferInfiniteQueryInput,
} from "@rspc/client";
import { createReactQueryHooks } from "@rspc/react";
import { MyPaginatedData, Procedures } from "./bindings";

export const rspc = createReactQueryHooks<Procedures>();

const client = createClient<Procedures>({
  transport: new NoOpTransport(),
});

function assert<T>(t: T) {}

// inferProcedureOutput
type A1 = inferProcedures<Procedures>;
assert<A1>(undefined as unknown as Procedures);
type B1 = inferProcedures<typeof client>;
assert<B1>(undefined as unknown as Procedures);
type C1 = inferProcedures<typeof rspc>;
assert<C1>(undefined as unknown as Procedures);

// inferQuery*
type A2 = inferQueryResult<Procedures, "noArgQuery">;
assert<A2>(undefined as unknown as string);
type B2 = inferQueryInput<Procedures, "noArgQuery">;
assert<B2>(undefined as unknown as never);
type C2 = inferQueryResult<Procedures, "singleArgQuery">;
assert<C2>(undefined as unknown as number);
type D2 = inferQueryInput<Procedures, "singleArgQuery">;
assert<D2>(undefined as unknown as number);

// inferMutation*
type A3 = inferMutationResult<Procedures, "noArgMutation">;
assert<A3>(undefined as unknown as string);
type B3 = inferMutationInput<Procedures, "noArgMutation">;
assert<B3>(undefined as unknown as never);
type C3 = inferMutationResult<Procedures, "singleArgMutation">;
assert<C3>(undefined as unknown as number);
type D3 = inferMutationInput<Procedures, "singleArgMutation">;
assert<D3>(undefined as unknown as number);

// inferSubscriptions*
type A4 = inferSubscriptionResult<Procedures, "noArgSubscription">;
assert<A4>(undefined as unknown as string);
type B4 = inferSubscriptionInput<Procedures, "noArgSubscription">;
assert<B4>(undefined as unknown as never);
type C4 = inferSubscriptionResult<Procedures, "singleArgSubscription">;
assert<C4>(undefined as unknown as boolean);
type D4 = inferSubscriptionInput<Procedures, "singleArgSubscription">;
assert<D4>(undefined as unknown as boolean);

// inferInfiniteQuery
type A5 = inferInfiniteQueries<Procedures>["key"];
assert<A5>(
  undefined as unknown as
    | "paginatedQueryOnlyCursor"
    | "paginatedQueryCursorAndArg"
);
type B5 = inferInfiniteQueryResult<Procedures, "paginatedQueryOnlyCursor">;
assert<B5>(undefined as unknown as MyPaginatedData);
type C5 = inferInfiniteQueryInput<Procedures, "paginatedQueryOnlyCursor">;
assert<C5>(undefined as unknown as never);
type D5 = inferInfiniteQueryResult<Procedures, "paginatedQueryCursorAndArg">;
assert<D5>(undefined as unknown as MyPaginatedData);
type E5 = inferInfiniteQueryInput<Procedures, "paginatedQueryCursorAndArg">;
assert<E5>(undefined as unknown as { my_param: number });
