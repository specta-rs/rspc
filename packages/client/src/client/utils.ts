import { ProcedureDef, ProceduresDef } from "../bindings";

export type GetProcedure<
  P extends ProcedureDef,
  K extends P["key"] & string
> = Extract<P, { key: K }>;

export type Queries<Procs extends ProceduresDef> = Procs["queries"];
export type Query<
  Procs extends ProceduresDef,
  K extends Queries<Procs>["key"]
> = GetProcedure<Queries<Procs>, K>;
export type Mutations<Procs extends ProceduresDef> = Procs["mutations"];
export type Mutation<
  Procs extends ProceduresDef,
  K extends Mutations<Procs>["key"]
> = GetProcedure<Mutations<Procs>, K>;
export type Subscriptions<Proces extends ProceduresDef> =
  Proces["subscriptions"];
export type Subscription<
  Procs extends ProceduresDef,
  K extends Subscriptions<Procs>["key"]
> = GetProcedure<Subscriptions<Procs>, K>;

export type Expand<T> = T extends infer O ? { [K in keyof O]: O[K] } : never;
export type TupleCond<T, Cond> = T extends Cond ? [] : [T];
// I think TS will only map over a union type if you use a conditional - @brendonovich
export type ProcedureKeyTuple<P extends ProcedureDef> = P extends ProcedureDef
  ? [key: P["key"], ...input: TupleCond<P["input"], null>]
  : never;
