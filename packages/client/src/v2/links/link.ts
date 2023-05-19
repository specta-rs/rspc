import { AlphaRSPCError } from "../error";

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
export interface Operation {
  // TODO: Optional on being a subscription?
  id: string; // TODO: Move back to being an int?

  type: "query" | "mutation" | "subscription" | "subscriptionStop"; // TODO: Derive this from Rust bindings
  input: unknown;
  path: string;
  context: OperationContext;
}

/**
 * TODO
 *
 * @internal
 */
export type LinkResult = {
  exec: (
    resolve: (result: any) => void,
    reject: (error: Error | AlphaRSPCError) => void
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
