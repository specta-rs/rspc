import { RSPCError, ProceduresLike, inferProcedures } from "..";
import {
  Unsubscribable,
  inferObservableValue,
  observableToPromise,
  share,
} from "../observable/index";

import { createChain } from "../links/internals/createChain";
import {
  OperationContext,
  OperationLink,
  TRPCClientRuntime,
  TRPCLink,
} from "../links/types";

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

interface CreateTRPCClientBaseOptions {
  /**
   * Data transformer
   * @link https://trpc.io/docs/data-transformers
   **/
  transformer?: ClientDataTransformerOptions;
}

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

/** @internal */
export type CreateTRPCClientOptions<TProcedures extends ProceduresLike> =
  | CreateTRPCClientBaseOptions & {
      links: TRPCLink<TProcedures>[];
    };

export class TRPCClient<TProcedures extends ProceduresLike> {
  private readonly links: OperationLink<TProcedures>[];
  public readonly runtime: TRPCClientRuntime;
  private requestId: number;

  constructor(opts: CreateTRPCClientOptions<TProcedures>) {
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
          // reject(TRPCClientError.from(err));
          throw new Error("TODO: Bruh"); // TODO
        });
    });

    return abortablePromise;
  }
  public query<
    // TODO: Generics
    TQueries extends any, // TProcedures["_def"]["queries"],
    TPath extends any, // string & keyof TQueries,
    TInput extends any // inferProcedureInput<TQueries[TPath]>
  >(path: TPath, input?: TInput, opts?: TRPCRequestOptions) {
    // type TOutput = inferProcedureOutput<TQueries[TPath]>;
    type TOutput = any; // TODO
    return this.requestAsPromise<TInput, TOutput>({
      type: "query",
      // @ts-expect-error: TODO: Fix this
      path,
      input: input as TInput,
      context: opts?.context,
      signal: opts?.signal,
    });
  }
  // public mutation<
  //   TMutations extends TProcedures["_def"]["mutations"],
  //   TPath extends string & keyof TMutations,
  //   TInput extends inferProcedureInput<TMutations[TPath]>
  // >(path: TPath, input?: TInput, opts?: TRPCRequestOptions) {
  //   type TOutput = inferProcedureOutput<TMutations[TPath]>;
  //   return this.requestAsPromise<TInput, TOutput>({
  //     type: "mutation",
  //     path,
  //     input: input as TInput,
  //     context: opts?.context,
  //     signal: opts?.signal,
  //   });
  // }
  // public subscription<
  //   TSubscriptions extends TProcedures["_def"]["subscriptions"],
  //   TPath extends string & keyof TSubscriptions,
  //   TOutput extends inferSubscriptionOutput<TProcedures, TPath>,
  //   TInput extends inferProcedureInput<TSubscriptions[TPath]>
  // >(
  //   path: TPath,
  //   input: TInput,
  //   opts: TRPCRequestOptions &
  //     Partial<TRPCSubscriptionObserver<TOutput, TRPCClientError<TProcedures>>>
  // ): Unsubscribable {
  //   const observable$ = this.$request<TInput, TOutput>({
  //     type: "subscription",
  //     path,
  //     input,
  //     context: opts?.context,
  //   });
  //   return observable$.subscribe({
  //     next(envelope) {
  //       if (envelope.result.type === "started") {
  //         opts.onStarted?.();
  //       } else if (envelope.result.type === "stopped") {
  //         opts.onStopped?.();
  //       } else {
  //         opts.onData?.((envelope.result as any).data);
  //       }
  //     },
  //     error(err) {
  //       opts.onError?.(err);
  //     },
  //     complete() {
  //       opts.onComplete?.();
  //     },
  //   });
  // }
}
