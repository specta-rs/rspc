import {
  inferProcedureInput,
  inferProcedureResult,
  inferProcedures,
  ProceduresLike,
} from "..";

/**
 * @deprecated Use `inferProcedureInput` instead
 */
export type inferQueryInput<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["queries"]["key"][0]
> = inferProcedureInput<inferProcedures<TProcedures>, "queries", T>;

/**
 * @deprecated Use `inferProcedureResult` instead
 */
export type inferQueryResult<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["queries"]["key"][0]
> = inferProcedureResult<inferProcedures<TProcedures>, "queries", T>;

/**
 * @deprecated Use `inferProcedureInput` instead
 */
export type inferMutationInput<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["mutations"]["key"][0]
> = inferProcedureInput<inferProcedures<TProcedures>, "mutations", T>;

/**
 * @deprecated Use `inferProcedureResult` instead
 */
export type inferMutationResult<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["mutations"]["key"][0]
> = inferProcedureResult<inferProcedures<TProcedures>, "mutations", T>;

/**
 * @deprecated Use `inferProcedureInput` instead
 */
export type inferSubscriptionInput<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["subscriptions"]["key"][0]
> = inferProcedureInput<inferProcedures<TProcedures>, "subscriptions", T>;

/**
 * @deprecated Use `inferProcedureResult` instead
 */
export type inferSubscriptionResult<
  TProcedures extends ProceduresLike,
  T extends inferProcedures<TProcedures>["subscriptions"]["key"][0]
> = inferProcedureResult<inferProcedures<TProcedures>, "subscriptions", T>;

// export type inferInfiniteQueries<TProcedures extends ProceduresLike> = Exclude<
//   Extract<inferProcedures<TProcedures>["queries"], { input: { cursor: any } }>,
//   { input: never }
// >;

// // TODO
// export type inferInfiniteQuery<
//   TProcedures extends ProceduresLike,
//   K extends inferInfiniteQueries<TProcedures>["key"]
// > = Extract<inferInfiniteQueries<TProcedures>, { key: K }>;

// // TODO
// type EmptyObjToNever<T> = keyof T extends never ? never : T;
// export type inferInfiniteQueryInput<
//   TProcedures extends ProceduresLike,
//   K extends inferInfiniteQueries<TProcedures>["key"]
// > = EmptyObjToNever<
//   Omit<inferInfiniteQuery<TProcedures, K>["input"], "cursor">
// >;

// // TODO
// export type inferInfiniteQueryResult<
//   TProcedures extends ProceduresLike,
//   K extends inferInfiniteQueries<TProcedures>["key"]
// > = inferInfiniteQuery<TProcedures, K>["result"];

// // TODO
// export type _inferInfiniteQueryProcedureHandlerInput<
//   TProcedures extends ProceduresLike,
//   K extends inferInfiniteQueries<TProcedures>["key"]
// > = inferInfiniteQueryInput<TProcedures, K> extends null
//   ? []
//   : [inferInfiniteQueryInput<TProcedures, K>];
