import type {
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  inferSubscriptionResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  SubscriptionOptions,
} from "..";
import {
  randomId,
  AlphaRSPCError,
  Link,
  OperationContext,
  Operation,
  LinkResult,
} from "../v2";

type KeyAndInput = [string] | [string, any];

type OperationOpts = {
  signal?: AbortSignal;
  context?: OperationContext;
};

// TODO
interface ClientArgs {
  links: Link[];
  onError?: (err: AlphaRSPCError) => void | Promise<void>;
}

export function initRspc<P extends ProceduresDef>(args: ClientArgs) {
  return new AlphaClient<P>(args);
}

// TODO: This will replace old client
export class AlphaClient<P extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private links: Link[];
  private subscriptionMap = new Map<string, (data: any) => void>();
  private onError?: (err: AlphaRSPCError) => void | Promise<void>;
  private mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput; // TODO: Do something so a single React.context can handle multiple of these

  constructor(args: ClientArgs) {
    if (args.links.length === 0) {
      throw new Error("Must provide at least one link");
    }

    this.links = args.links;
    this.subscriptionMap = new Map();
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
          id: 0,
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
          id: 0,
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

  // TODO: AbortController's with subscriptions?
  // TODO: Redesign this, i'm sure it probably has race conditions but it works for now
  addSubscription<
    K extends P["subscriptions"]["key"] & string,
    TData = inferSubscriptionResult<P, K>
  >(
    keyAndInput: [K, ..._inferProcedureHandlerInput<P, "subscriptions", K>],
    opts: SubscriptionOptions<TData>
  ): () => void {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;

      let subscriptionId = randomId();
      let unsubscribed = false;

      const cleanup = () => {
        this.subscriptionMap?.delete(subscriptionId);
        if (subscriptionId) {
          // @ts-expect-error // TODO
          this.transport.doRequest(
            "subscriptionStop",
            undefined!,
            subscriptionId
          );
        }
      };

      // @ts-expect-error // TODO
      this.transport.doRequest("subscription", keyAndInput2[0], [
        subscriptionId,
        keyAndInput2[1],
      ]);

      if (opts.onStarted) opts.onStarted();
      this.subscriptionMap?.set(subscriptionId, opts.onData);

      return () => {
        unsubscribed = true;
        cleanup();
      };
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
  for (const link of links) {
    const result = link({
      op,
      next: () => prevLinkResult,
    });
    prevLinkResult = result;
  }

  return prevLinkResult;
}
