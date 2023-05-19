import type {
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  inferProcedureResult,
} from "..";
import {
  AlphaRSPCError,
  Link,
  OperationContext,
  Operation,
  LinkResult,
} from "../v2";

// TODO
export interface SubscriptionOptions<TOutput> {
  // onStarted?: () => void;
  onData: (data: TOutput) => void;
  // TODO: Probs remove `| Error` here
  onError?: (err: AlphaRSPCError | Error) => void;
}

type KeyAndInput = [string] | [string, any];

type OperationOpts = {
  signal?: AbortSignal;
  context?: OperationContext;
  // skipBatch?: boolean; // TODO: Make this work + add this to React
};

// TODO
interface ClientArgs {
  links: Link[];
  onError?: (err: AlphaRSPCError) => void | Promise<void>;
}

export function initRspc<P extends ProceduresDef>(args: ClientArgs) {
  return new AlphaClient<P>(args);
}

const generateRandomId = () => Math.random().toString(36).slice(2);

// TODO: This will replace old client
export class AlphaClient<P extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private links: Link[];
  private onError?: (err: AlphaRSPCError) => void | Promise<void>;
  private mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput; // TODO: Do something so a single React.context can handle multiple of these

  constructor(args: ClientArgs) {
    if (args.links.length === 0) {
      throw new Error("Must provide at least one link");
    }

    this.links = args.links;
    this.onError = args.onError;
  }

  async query<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: OperationOpts
  ): Promise<inferQueryResult<P, K>> {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;

      const result = exec(
        {
          id: generateRandomId(),
          type: "query",
          input: keyAndInput2[1],
          path: keyAndInput2[0],
          context: opts?.context || {},
        },
        this.links
      );
      opts?.signal?.addEventListener("abort", result.abort);

      return await new Promise(result.exec);
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      throw err;
    }
  }

  async mutation<K extends P["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "mutations", K>
    ],
    opts?: OperationOpts
  ): Promise<inferMutationResult<P, K>> {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;

      const result = exec(
        {
          id: generateRandomId(),
          type: "mutation",
          input: keyAndInput2[1],
          path: keyAndInput2[0],
          context: opts?.context || {},
        },
        this.links
      );
      opts?.signal?.addEventListener("abort", result.abort);

      return await new Promise(result.exec);
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      throw err;
    }
  }

  // TODO: Handle resubscribing if the subscription crashes similar to what Tanstack Query does
  addSubscription<
    K extends P["subscriptions"]["key"] & string,
    TData = inferProcedureResult<P, "subscriptions", K>
  >(
    keyAndInput: [K, ..._inferProcedureHandlerInput<P, "subscriptions", K>],
    opts: SubscriptionOptions<TData> & { context?: OperationContext }
  ): () => void {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;

      const result = exec(
        {
          id: generateRandomId(),
          type: "subscription",
          input: keyAndInput2[1],
          path: keyAndInput2[0],
          context: opts?.context || {},
        },
        this.links
      );

      result.exec(
        (data) => opts?.onData(data),
        (err) => opts?.onError?.(err)
      );
      return result.abort;
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      return () => {};
    }
  }

  // TODO: Remove this once middleware system is in place
  dangerouslyHookIntoInternals<P2 extends ProceduresDef = P>(opts?: {
    mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput;
  }): AlphaClient<P2> {
    this.mapQueryKey = opts?.mapQueryKey;
    return this as any;
  }
}

function exec(op: Operation, links: Link[]) {
  if (!links[0]) throw new Error("No links provided");

  let prevLinkResult: LinkResult = {
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
