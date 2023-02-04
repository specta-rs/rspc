// @ts-nocheck: TODO: Bruh

// import { AnyRouter, DataTransformer } from '@trpc/server';
// import { Observable, Observer } from '@trpc/server/observable';
// import { TRPCResultMessage, TRPCSuccessResponse } from '@trpc/server/rpc';

import { RSPCError } from "../interop/error";
import { Observable, Observer } from "../internals/observable";
import { OperationContext } from "..";

type AnyRouter = any;
type DataTransformer = any;
type TRPCResultMessage<T> = any;
type TRPCSuccessResponse<T> = any;

/**
 * @internal
 */
export type CancelFn = () => void;

/**
 * @internal
 */
export type PromiseAndCancel<TValue> = {
  promise: Promise<TValue>;
  cancel: CancelFn;
};

/**
 * @internal
 */
export type LegacyOperation<TInput = unknown> = {
  id: number;
  type: "query" | "mutation" | "subscription";
  input: TInput;
  path: string;
  context: OperationContext;
};

/**
 * @internal
 */
// export type HTTPHeaders = Record<string, string | string[] | undefined>;

/**
 * The default `fetch` implementation has an overloaded signature. By convention this library
 * only uses the overload taking a string and options object.
 */
export type TRPCFetch = (
  url: string,
  options?: RequestInit
) => Promise<Response>;

export type OnErrorFunction = (opts: {
  error: RSPCError;
  type: string; // TODO: ProcedureType | "unknown";
  path: string | undefined;
  // req: TRequest;
  input: unknown;
  ctx: undefined | any; // TODO: inferRouterContext<TRouter>;
}) => void;

export interface TRPCClientRuntime {
  transformer: DataTransformer;
  onError?: OnErrorFunction;
}

/**
 * @internal
 */
export interface OperationResultEnvelope<TOutput> {
  result:
    | TRPCSuccessResponse<TOutput>["result"]
    | TRPCResultMessage<TOutput>["result"];
  context?: OperationContext;
}

/**
 * @internal
 */
export type OperationResultObservable<
  TRouter extends AnyRouter,
  TOutput
> = Observable<OperationResultEnvelope<TOutput>, RSPCError>;

/**
 * @internal
 */
export type OperationResultObserver<
  TRouter extends AnyRouter,
  TOutput
> = Observer<OperationResultEnvelope<TOutput>, RSPCError>;

/**
 * @internal
 */
export type OperationLink<
  TRouter extends AnyRouter,
  TInput = unknown,
  TOutput = unknown
> = (opts: {
  op: LegacyOperation<TInput>;
  next: (
    op: LegacyOperation<TInput>
  ) => OperationResultObservable<TRouter, TOutput>;
}) => OperationResultObservable<TRouter, TOutput>;

/**
 * @public
 */
export type TRPCLink<TRouter extends AnyRouter> = (
  opts: TRPCClientRuntime
) => OperationLink<TRouter>;
