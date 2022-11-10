import {
  RSPCError,
  ProceduresLike,
  inferProcedures,
  _inferProcedureHandlerInput,
  ProcedureDef,
} from ".";
import {
  Unsubscribable,
  inferObservableValue,
  observableToPromise,
  share,
} from "./internals/observable/index";

import { createChain } from "./links/internals/createChain";
import {
  OnErrorFunction,
  OperationContext,
  OperationLink,
  TRPCClientRuntime,
  TRPCLink,
} from "./links/types";

/**
 * @public
 */
export type DataTransformer = {
  serialize(object: any): any;
  deserialize(object: any): any;
};

/**
 * @public
 */
export type CombinedDataTransformer = {
  input: DataTransformer;
  output: DataTransformer;
};

/**
 * @public
 */
export type CombinedDataTransformerClient = {
  input: Pick<DataTransformer, "serialize">;
  output: Pick<DataTransformer, "deserialize">;
};

/**
 * @public
 */
export type DataTransformerOptions = DataTransformer | CombinedDataTransformer;

/**
 * @public
 */
export type ClientDataTransformerOptions =
  | DataTransformer
  | CombinedDataTransformerClient;

type TRPCType = "subscription" | "query" | "mutation";
export interface TRPCRequestOptions {
  /**
   * Pass additional context to links
   */
  context?: OperationContext;
  signal?: AbortSignal;
}

export interface TRPCSubscriptionObserver<TValue, TError> {
  onStarted: () => void;
  onData: (value: TValue) => void;
  onError: (err: TError) => void;
  onStopped: () => void;
  onComplete: () => void;
}

export interface ClientArgs {
  /**
   * Data transformer
   * @link https://trpc.io/docs/data-transformers
   **/
  transformer?: ClientDataTransformerOptions;
  /**
   * TODO
   * @link https://rspc.dev/todo
   **/
  links: TRPCLink<any>[];
  /**
   * TODO
   * @link https://rspc.dev/todo
   **/
  onError?: OnErrorFunction;
}

// TODO: Probs move these into Typescript helpers
type Procedure<T extends ProcedureDef, K extends T["key"]> = Extract<
  T,
  { key: K }
>;

function getTransformer(opts: ClientArgs): DataTransformer {
  if (!opts.transformer)
    return {
      serialize: (data) => data,
      deserialize: (data) => data,
    };
  if ("input" in opts.transformer)
    return {
      serialize: opts.transformer.input.serialize,
      deserialize: opts.transformer.output.deserialize,
    };
  return opts.transformer;
}

export function createClient<
  TBaseProceduresLike extends ProceduresLike,
  TQueries extends ProcedureDef = inferProcedures<TBaseProceduresLike>["queries"],
  TMutations extends ProcedureDef = inferProcedures<TBaseProceduresLike>["mutations"],
  TSubscriptions extends ProcedureDef = inferProcedures<TBaseProceduresLike>["subscriptions"]
>(opts: ClientArgs) {
  type TQuery<K extends TQueries["key"] & string> = Extract<
    TQueries,
    { key: K }
  >;
  type TMutation<K extends TMutations["key"] & string> = Extract<
    TMutations,
    { key: K }
  >;
  type TSubscription<K extends TSubscriptions["key"] & string> = Extract<
    TSubscriptions,
    { key: K }
  >;
  const runtime: TRPCClientRuntime = {
    transformer: getTransformer(opts),
    onError: opts.onError,
  };
  const links: OperationLink<any>[] = opts.links.map((link) => link(runtime));
  let requestId = 0;

  function $request<TInput = unknown, TOutput = unknown>({
    type,
    input,
    path,
    context = {},
  }: {
    type: TRPCType;
    input: TInput;
    path: string;
    context?: OperationContext;
  }) {
    const chain$ = createChain<inferProcedures<any>, TInput, TOutput>({
      links: links as OperationLink<any, any, any>[],
      op: {
        id: ++requestId,
        type,
        path,
        input,
        context,
      },
    });
    return chain$.pipe(share());
  }

  function requestAsPromise<TInput = unknown, TOutput = unknown>(opts: {
    type: TRPCType;
    input: TInput;
    path: string;
    context?: OperationContext;
    signal?: AbortSignal;
  }): Promise<TOutput> {
    const req$ = $request<TInput, TOutput>(opts);
    type TValue = inferObservableValue<typeof req$>;
    const { promise, abort } = observableToPromise<TValue>(req$);

    const abortablePromise = new Promise<TOutput>((resolve, reject) => {
      opts.signal?.addEventListener("abort", abort);

      promise
        .then((envelope) => {
          resolve((envelope.result as any).data);
        })
        .catch((err) => {
          reject(RSPCError.from(err));
        });
    });

    return abortablePromise;
  }

  function query<K extends TQueries["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: TQuery<K>["input"] extends null ? [] : [TQuery<K>["input"]]
    ],
    opts?: TRPCRequestOptions
  ): Promise<TQuery<K>["result"]> {
    return requestAsPromise<any, any>({
      type: "query",
      path: keyAndInput[0],
      input: keyAndInput[1],
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  function mutation<K extends TMutations["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: TMutation<K>["input"] extends null
        ? []
        : [TMutation<K>["input"]]
    ],
    opts?: TRPCRequestOptions
  ): Promise<TMutation<K>["result"]> {
    return requestAsPromise<any, any>({
      type: "mutation",
      path: keyAndInput[0],
      input: keyAndInput[1],
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  function subscription<K extends TSubscriptions["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: TSubscription<K>["input"] extends null
        ? []
        : [TSubscription<K>["input"]]
    ],
    opts: TRPCRequestOptions &
      Partial<TRPCSubscriptionObserver<TSubscription<K>["result"], RSPCError>>
  ): Unsubscribable {
    const observable$ = $request<any, any>({
      type: "subscription",
      path: keyAndInput[0],
      input: keyAndInput[1],
      context: opts?.context,
    });
    return observable$.subscribe({
      next(envelope) {
        if (envelope.result.type === "started") {
          opts.onStarted?.();
        } else if (envelope.result.type === "stopped") {
          opts.onStopped?.();
        } else {
          opts.onData?.((envelope.result as any).data);
        }
      },
      error(err) {
        opts.onError?.(err);
      },
      complete() {
        opts.onComplete?.();
      },
    });
  }

  return {
    _rspc_def: undefined as unknown as {
      queries: TQueries;
      mutations: TMutations;
      subscriptions: TSubscriptions;
    },
    query,
    mutation,
    subscription,
  };
}

export type Client<
  TBaseProceduresLike extends ProceduresLike,
  TQueries extends ProcedureDef = inferProcedures<TBaseProceduresLike>["queries"],
  TMutations extends ProcedureDef = inferProcedures<TBaseProceduresLike>["mutations"],
  TSubscriptions extends ProcedureDef = inferProcedures<TBaseProceduresLike>["subscriptions"]
> = ReturnType<
  typeof createClient<TBaseProceduresLike, TQueries, TMutations, TSubscriptions>
>;
