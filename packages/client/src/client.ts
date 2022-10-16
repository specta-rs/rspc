// TODO: Redo this entire system when links are introduced
import {
  RSPCError,
  ProceduresLike,
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  inferProcedures,
  inferSubscriptionResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  inferClientProxy,
  ClientOperationProxyRenames,
  ClientOperationProxyKey,
} from ".";
import { randomId, Transport } from "./transport";
import { createProxy } from "./utils/createProxy";

// TODO
export interface SubscriptionOptions<TOutput> {
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

// TODO
export interface ClientArgs {
  transport: Transport;
  onError?: (err: RSPCError) => void | Promise<void>;
}
type RSPCClient<TProcedures extends ProceduresDef> =
  inferClientProxy<TProcedures> & Client<TProcedures>

// TODO
export function createClient<TProcedures extends ProceduresDef>(
  args: ClientArgs
): RSPCClient<TProcedures> {
  let client = new Client(args);
  let proxy = createProxy(({ keys,params }) => {
    // Return early if a single key is given
    if (keys.length === 1) {
      switch (keys[0]) {
        case "query": return client.query(params[0] as any)
        case "mutation": return client.mutation(params[0] as any)
        case "addSubscription": return client.addSubscription(params[0] as any, params[1] as any)
      }
    }

    const _keys = [...keys];
    const caller = _keys.pop()! as ClientOperationProxyKey;
    const methodName = ClientOperationProxyRenames[caller];
    const key = _keys.join('.');

    // Assuming that always the last params is always an object
    // representing the extra options.
    let opts = params.pop();

    // Add it back if it's not an object
    if (typeof opts !== "object" && !Array.isArray(opts)) {
      params.push(opts);
      opts = undefined;
    }

    console.log("Running", { key, params: [...params], opts })

    return (client[methodName] as any)([key, ...params] as any, opts)
  })

  return proxy as any
}

// TODO
export class Client<TProcedures extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private transport: Transport;
  private subscriptionMap = new Map<string, (data: any) => void>();
  private onError?: (err: RSPCError) => void | Promise<void>;

  constructor(args: ClientArgs) {
    this.transport = args.transport;
    this.transport.clientSubscriptionCallback = (id, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
    this.onError = args.onError;
  }

  async query<K extends TProcedures["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ]
  ): Promise<inferQueryResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest(
        "query",
        keyAndInput[0],
        keyAndInput[1]
      );
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }
      throw err;
    }
  }

  async mutation<K extends TProcedures["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "mutations", K>
    ]
  ): Promise<inferMutationResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest(
        "mutation",
        keyAndInput[0],
        keyAndInput[1]
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
    K extends TProcedures["subscriptions"]["key"] & string,
    TData = inferSubscriptionResult<TProcedures, K>
  >(
    keyAndInput: [
      K,
      _inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: SubscriptionOptions<TData>
  ): () => void {
    try {
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

      this.transport.doRequest("subscription", keyAndInput[0], [
        subscriptionId,
        keyAndInput[1],
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
}
