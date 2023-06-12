import { RSPCError } from "..";
import { Request as RspcRequest } from "..";

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
export type LinkResult = {
  exec: (
    resolve: (result: any) => void,
    reject: (error: Error | RSPCError) => void
  ) => void;
  abort: () => void;
};

/**
 * The argument to a link. Contains information about the current operation and a function to call the next link.
 *
 * @internal
 */
export interface LinkOperation {
  op: Operation;
  next(op: { op: Operation }): LinkResult;
}

/**
 * TODO
 *
 * @internal
 */
export type Link = (p: LinkOperation) => LinkResult;
