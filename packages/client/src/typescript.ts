import { inferProcedures, ProceduresDef, ProceduresLike } from ".";

// TODO: This should come from Rust via Specta
export type OperationType =
  | "query"
  | "mutation"
  | "subscription"
  | "subscriptionStop";

/**
 * TODO: Docs
 */
export type inferProcedureKey<
  TProcedures extends ProceduresLike,
  TOperation extends keyof ProceduresDef
> = inferProcedures<TProcedures>[TOperation]["key"];

// TODO
export type inferProcedure<
  TProcedures extends ProceduresLike,
  TOperation extends keyof ProceduresDef,
  K extends inferProcedureKey<TProcedures, TOperation>
> = Extract<inferProcedures<TProcedures>[TOperation], { key: K }>;

// TODO
export type inferProcedureInput<
  TProcedures extends ProceduresLike,
  TOperation extends keyof ProceduresDef,
  K extends inferProcedureKey<TProcedures, TOperation>
> = inferProcedure<TProcedures, TOperation, K>["input"];

// TODO
export type inferProcedureResult<
  TProcedures extends ProceduresLike,
  TOperation extends keyof ProceduresDef,
  K extends inferProcedureKey<TProcedures, TOperation>
> = inferProcedure<TProcedures, TOperation, K>["result"];
