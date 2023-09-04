import {
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  inferQueryError,
  inferQuery,
  ProcedureDef,
  inferMutation,
  inferMutationError,
  inferSubscription,
} from ".";
import { Link, OperationContext, Operation, LinkResult } from ".";

// TODO
export interface SubscriptionOptions<P extends ProcedureDef> {
  // onStarted?: () => void;
  onData: (data: P["result"]) => void;
  // TODO: Probs remove `| Error` here
  onError?: (err: P["error"] | Error) => void;
}

type KeyAndInput = [string] | [string, any];

type OperationOpts = {
  signal?: AbortSignal;
  context?: OperationContext;
  // skipBatch?: boolean; // TODO: Make this work + add this to React
};

// TODO
interface ClientArgs<P extends ProceduresDef> {
  links: Link<P>[];
  onError?: OnErrorHandler<P>;
}

export function initRspc<P extends ProceduresDef>(args: ClientArgs<P>) {
  return new AlphaClient<P>(args);
}

export type Result<TOk, TErr> =
  | { status: "ok"; data: TOk }
  | { status: "error"; error: TErr };

type OnErrorHandler<P extends ProceduresDef> = (
  err: P[keyof ProceduresDef]["error"]
) => void | Promise<void>;

export class AlphaClient<P extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private links: Link<P>[];
  private onError?: OnErrorHandler<P>;
  private mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput; // TODO: Do something so a single React.context can handle multiple of these

  constructor(args: ClientArgs<P>) {
    if (args.links.length === 0) {
      throw new Error("Must provide at least one link");
    }

    this.links = args.links;
    this.onError = args.onError;
  }

  query<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: OperationOpts
  ): Promise<Result<inferQueryResult<P, K>, inferQueryError<P, K>>> {
    const keyAndInput2 = this.mapQueryKey
      ? this.mapQueryKey(keyAndInput as any)
      : keyAndInput;

    const result = exec<P>(
      {
        method: "query",
        input: keyAndInput2[1],
        path: keyAndInput2[0],
        context: opts?.context || {},
      },
      this.links
    );

    opts?.signal?.addEventListener("abort", result.abort);

    return new Promise((res) => {
      result.exec(
        (data) => res({ status: "ok", data }),
        (error) => {
          this.onError?.(error);

          res({ status: "error", error });
        }
      );
    });
  }

  mutation<K extends P["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "mutations", K>
    ],
    opts?: OperationOpts
  ): Promise<Result<inferMutationResult<P, K>, inferMutationError<P, K>>> {
    const keyAndInput2 = this.mapQueryKey
      ? this.mapQueryKey(keyAndInput as any)
      : keyAndInput;

    const result = exec<P>(
      {
        method: "mutation",
        input: keyAndInput2[1],
        path: keyAndInput2[0],
        context: opts?.context || {},
      },
      this.links
    );
    opts?.signal?.addEventListener("abort", result.abort);

    return new Promise((res) => {
      result.exec(
        (data) => res({ status: "ok", data }),
        (error) => {
          this.onError?.(error);

          res({ status: "error", error });
        }
      );
    });
  }

  // TODO: Handle resubscribing if the subscription crashes similar to what Tanstack Query does
  addSubscription<K extends P["subscriptions"]["key"] & string>(
    keyAndInput: [K, ..._inferProcedureHandlerInput<P, "subscriptions", K>],
    opts: SubscriptionOptions<inferSubscription<P, K>> & {
      context?: OperationContext;
    }
  ): () => void {
    const keyAndInput2 = this.mapQueryKey
      ? this.mapQueryKey(keyAndInput as any)
      : keyAndInput;

    const result = exec<P>(
      {
        method: "subscription",
        input: keyAndInput2[1],
        path: keyAndInput2[0],
        context: opts?.context || {},
      },
      this.links
    );

    result.exec(
      (data) => opts?.onData(data),
      (error) => {
        this.onError?.(error);

        opts?.onError?.(error);
      }
    );

    return result.abort;
  }

  // TODO: Remove this once middleware system is in place
  dangerouslyHookIntoInternals<P2 extends ProceduresDef = P>(opts?: {
    mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput;
  }): AlphaClient<P2> {
    this.mapQueryKey = opts?.mapQueryKey;
    return this as any;
  }
}

function exec<P extends ProceduresDef>(op: Operation, links: Link<P>[]) {
  if (!links[0]) throw new Error("No links provided");

  let prevLinkResult: LinkResult<P> = {
    exec: () => {
      throw new Error(
        "rspc: no terminating link was attached! Did you forget to add a 'httpLink' or 'wsLink' link?"
      );
    },
    abort: () => {},
  };

  for (var linkIndex = 0; linkIndex < links.length; linkIndex++) {
    const link = links[links.length - linkIndex - 1]!;
    const result = link({
      op,
      next: () => prevLinkResult,
    });
    prevLinkResult = result;
  }

  return prevLinkResult;
}
