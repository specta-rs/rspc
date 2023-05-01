import {
  randomId,
  Transport,
  RSPCError,
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  inferSubscriptionResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  ClientArgs,
  SubscriptionOptions,
} from "..";

type KeyAndInput = [string] | [string, any];

// TODO: This will replace old client
export class AlphaClient<P extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private transport: Transport;
  private subscriptionMap = new Map<string, (data: any) => void>();
  private onError?: (err: RSPCError) => void | Promise<void>;
  private mapQueryKey?: (keyAndInput: KeyAndInput) => KeyAndInput;

  constructor(args: ClientArgs) {
    this.transport = args.transport;
    this.transport.clientSubscriptionCallback = (id, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
    this.onError = args.onError;
  }

  async query<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "queries", K>
    ]
  ): Promise<inferQueryResult<P, K>> {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;
      return await this.transport.doRequest(
        "query",
        keyAndInput2[0],
        keyAndInput2[1]
      );
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }
      throw err;
    }
  }

  async mutation<K extends P["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "mutations", K>
    ]
  ): Promise<inferMutationResult<P, K>> {
    try {
      const keyAndInput2 = this.mapQueryKey
        ? this.mapQueryKey(keyAndInput as any)
        : keyAndInput;
      return await this.transport.doRequest(
        "mutation",
        keyAndInput2[0],
        keyAndInput2[1]
      );
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }
      throw err;
    }
  }

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
          this.transport.doRequest(
            "subscriptionStop",
            undefined!,
            subscriptionId
          );
        }
      };

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
        this.onError(err as RSPCError);
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

  // TODO: Remove this?
  dangerouslyClone() {
    const clone = Object.assign({}, this);
    Object.setPrototypeOf(clone, AlphaClient.prototype);
    clone.transport = this.transport;
    clone.onError = this.onError;
    clone.mapQueryKey = this.mapQueryKey;
    return clone as typeof this;
  }
}
