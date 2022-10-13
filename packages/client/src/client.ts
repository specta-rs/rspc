import {
  RSPCError,
  ProceduresLike,
  inferProcedures,
  ProceduresDef,
  inferQueryResult,
  _inferProcedureHandlerInput,
  inferQueryInput,
  inferProcedureInput,
  inferProcedureResult,
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

export function createClient<TProceduresLike extends ProceduresLike>(
  opts: ClientArgs
) {
  const client = new Client<inferProcedures<TProceduresLike>>(opts);
  return client;
}

export class Client<TProcedures extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private readonly links: OperationLink<TProcedures>[];
  public readonly runtime: TRPCClientRuntime;
  private requestId: number;

  constructor(opts: ClientArgs) {
    this.requestId = 0;

    function getTransformer(): DataTransformer {
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

    this.runtime = {
      transformer: getTransformer(),
      onError: opts.onError,
    };
    // Initialize the links
    this.links = opts.links.map((link) => link(this.runtime));
  }

  private $request<TInput = unknown, TOutput = unknown>({
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
    const chain$ = createChain<inferProcedures<TProcedures>, TInput, TOutput>({
      links: this.links as OperationLink<any, any, any>[],
      op: {
        id: ++this.requestId,
        type,
        path,
        input,
        context,
      },
    });
    return chain$.pipe(share());
  }
  private requestAsPromise<TInput = unknown, TOutput = unknown>(opts: {
    type: TRPCType;
    input: TInput;
    path: string;
    context?: OperationContext;
    signal?: AbortSignal;
  }): Promise<TOutput> {
    const req$ = this.$request<TInput, TOutput>(opts);
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

  public query<K extends TProcedures["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ],
    opts?: TRPCRequestOptions
  ): Promise<inferProcedureResult<TProcedures, "queries", K>> {
    return this.requestAsPromise<any, any>({
      type: "query",
      path: keyAndInput[0],
      input: keyAndInput[1],
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  public mutation<K extends TProcedures["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "mutations", K>
    ],
    opts?: TRPCRequestOptions
  ): Promise<inferProcedureResult<TProcedures, "mutations", K>> {
    return this.requestAsPromise<any, any>({
      type: "mutation",
      path: keyAndInput[0],
      input: keyAndInput[1],
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  public subscription<K extends TProcedures["subscriptions"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: TRPCRequestOptions &
      Partial<
        TRPCSubscriptionObserver<
          inferProcedureResult<TProcedures, "subscriptions", K>,
          RSPCError
        >
      >
  ): Unsubscribable {
    const observable$ = this.$request<any, any>({
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
}
