import * as rspc from "../bindings";
import { ProceduresDef } from "../bindings";

/**
 * A map of data that can be used by links to store metadata about the current operation.
 * This allows links to communicate with each other.
 *
 * @internal
 */
export type OperationContext = Record<string, unknown>;

/**
 * TODO
 *
 * @internal
 */
export type Operation = {
  method: "query" | "mutation" | "subscription";
  path: string;
  input: any | null;
  context: OperationContext;
};

/**
 * TODO
 *
 * @internal
 */
export type LinkResult<P extends ProceduresDef> = {
  exec: (
    resolve: (result: P[keyof ProceduresDef]["result"]) => void,
    reject: (error: P[keyof ProceduresDef]["error"] | rspc.Error) => void
  ) => void;
  abort: () => void;
};

/**
 * The argument to a link. Contains information about the current operation and a function to call the next link.
 *
 * @internal
 */
export interface LinkOperation<P extends ProceduresDef> {
  op: Operation;
  next(op: { op: Operation }): LinkResult<P>;
}

/**
 * TODO
 *
 * @internal
 */
export type Link<P extends ProceduresDef> = (
  p: LinkOperation<P>
) => LinkResult<P>;
