// This file is a unit test for the Typescript inference helpers

// Run `cargo run --example typescript_test` before checking this.

import {
  inferBindingsType,
  inferMutationInput,
  inferMutationResult,
  inferQueryInput,
  inferQueryResult,
  inferSubscriptionInput,
  inferSubscriptionResult,
} from "@rspc/client";
import { Operations } from "../../bindings";
import { fetchClient, rspc } from "./react";

function assert<T>(t: T) {}

// inferProcedureOutput
type A1 = inferBindingsType<Operations>;
assert<A1>(undefined as Operations);
type B1 = inferBindingsType<typeof fetchClient>;
assert<B1>(undefined as Operations);
type C1 = inferBindingsType<typeof rspc>;
assert<C1>(undefined as Operations);

// inferQuery*
type A2 = inferQueryResult<Operations, "noArgQuery">;
assert<A2>(undefined as string);
type B2 = inferQueryInput<Operations, "noArgQuery">;
assert<B2>(undefined as undefined);
type C2 = inferQueryResult<Operations, "singleArgQuery">;
assert<C2>(undefined as number);
type D2 = inferQueryInput<Operations, "singleArgQuery">;
assert<D2>(undefined as number);

// inferMutation*
type A3 = inferMutationResult<Operations, "noArgMutation">;
assert<A3>(undefined as string);
type B3 = inferMutationInput<Operations, "noArgMutation">;
assert<B3>(undefined as undefined);
type C3 = inferMutationResult<Operations, "singleArgMutation">;
assert<C3>(undefined as number);
type D3 = inferMutationInput<Operations, "singleArgMutation">;
assert<D3>(undefined as number);

// inferSubscriptions*
type A4 = inferSubscriptionResult<Operations, "noArgSubscription">;
assert<A4>(undefined as string);
type B4 = inferSubscriptionInput<Operations, "noArgSubscription">;
assert<B4>(undefined as undefined);
type C4 = inferSubscriptionResult<Operations, "singleArgSubscription">;
assert<C4>(undefined as boolean);
type D4 = inferSubscriptionInput<Operations, "singleArgSubscription">;
assert<D4>(undefined as boolean);
