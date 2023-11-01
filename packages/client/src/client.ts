import {
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  inferQueryError,
  ProcedureDef,
  inferMutationError,
  inferSubscription,
} from ".";
import { Link, OperationContext, Operation, LinkResult } from ".";
import * as rspc from "./bindings";

// TODO
export interface SubscriptionOptions<P extends ProcedureDef> {
  // onStarted?: () => void;
  onData: (data: P["result"]) => void;
  // TODO: Probs remove `| Error` here
  onError?: (err: P["error"] | rspc.Error) => void;
}

type KeyAndInput = [string] | [string, any];

type OperationOpts = {
  signal?: AbortSignal;
  context?: OperationContext;
  // skipBatch?: boolean; // TODO: Make this work + add this to React
};

// TODO
export interface ClientArgs<P extends ProceduresDef> {
  links: Link<P>[];
  onError?: OnErrorHandler<P>;
  root: Root<P>;
}

export class Root<P extends ProceduresDef> {
  public _rspc_def: P = undefined!;
  parent?: Root<any>;
  private _mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput;

  createChild<P2 extends ProceduresDef>(args: {
    mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput;
  }) {
    const root = new Root<P2>();
    root.parent = this;

    if (args.mapQueryKey) root._mapQueryKey = args.mapQueryKey;

    return root;
  }

  mapQueryKey(keyAndInput: KeyAndInput): KeyAndInput {
    const afterParentApplied =
      this.parent?.mapQueryKey(keyAndInput) ?? keyAndInput;
    return this._mapQueryKey?.(afterParentApplied!) ?? afterParentApplied;
  }
}

export function createRSPCClient<P extends ProceduresDef>(
  args: Omit<ClientArgs<P>, "root">
) {
  return new Client<P>({ ...args, root: new Root<P>() });
}

export type Result<TOk, TErr> =
  | { status: "ok"; data: TOk }
  | { status: "error"; error: TErr };

type OnErrorHandler<P extends ProceduresDef> = (
  err: P[keyof ProceduresDef]["error"]
) => void | Promise<void>;

export class Client<P extends ProceduresDef> {
  _root: Root<P>;
  public _rspc_def: ProceduresDef = undefined!;
  private links: Link<any>[];
  private onError?: OnErrorHandler<any>;

  constructor(args: ClientArgs<P>) {
    if (args.links.length === 0) {
      throw new Error("Must provide at least one link");
    }

    this.links = args.links;
    this.onError = args.onError;
    this._root = args.root;
  }

  query<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: OperationOpts
  ): Promise<Result<inferQueryResult<P, K>, inferQueryError<P, K>>> {
    const keyAndInput2 = this._root.mapQueryKey(keyAndInput as any);

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
    const keyAndInput2 = this._root.mapQueryKey(keyAndInput as any);

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
    const keyAndInput2 = this._root.mapQueryKey(keyAndInput as any);

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

  createChild<P2 extends ProceduresDef = P>(opts: {
    root: Root<P2>;
  }): Client<P2> {
    if (opts.root.parent !== this._root)
      throw new Error(
        "Child clients must have a root that is a child of their parent's root"
      );

    const client = new Client<P2>({
      root: opts.root,
      links: this.links,
      onError: this.onError,
    });

    return client;
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
