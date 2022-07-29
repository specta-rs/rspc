import { Transport } from "./transport";

export type OperationType =
  | "query"
  | "mutation"
  | "subscriptionAdd"
  | "subscriptionRemove";

export type OperationKey = [string] | [string, /* args */ any];

export type OperationsDef = {
  queries: { key: OperationKey; result: any };
  mutations: { key: OperationKey; result: any };
  subscriptions: { key: OperationKey; result: any };
};

export interface ClientTransformer {
  serialize(type: OperationType, key: OperationKey): OperationKey;
  deserialize(type: OperationType, key: OperationKey, data: any): any;
}
export interface ClientArgs {
  transport: Transport;
  transformer?: ClientTransformer;
}

export function createClient<T extends OperationsDef>(
  args: ClientArgs
): Client<T> {
  return new Client(args);
}

export class Client<T extends OperationsDef> {
  private transport: Transport;
  private subscriptionMap = new Map<string, (data: any) => void>();

  constructor(args: ClientArgs) {
    this.transport = args.transport;
    this.transport.transformer = args.transformer;
    this.transport.clientSubscriptionCallback = (id, key, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
  }

  async query<K extends T["queries"]["key"]>(
    key: K
  ): Promise<Extract<T["queries"], { key: K }>["result"]> {
    return await this.transport.doRequest("query", key);
  }

  async mutation<K extends T["mutations"]["key"][0]>(
    key: [K, Extract<T["mutations"], { key: [K, any] }>["key"][1]]
  ): Promise<Extract<T["mutations"], { key: K }>["result"]> {
    return await this.transport.doRequest("mutation", key);
  }

  // TODO: Redesign this, i'm sure it probably has race conditions but it functions for now
  addSubscription<K extends T["subscriptions"]["key"][0]>(
    key: Extract<
      T["subscriptions"]["key"],
      { key: [K, any] }
    >[1] extends undefined
      ? [K]
      : [K, Extract<T["subscriptions"]["key"], { key: [K, any] }>[1]],
    options: {
      onNext(msg: Extract<T["subscriptions"], { key: [K, any] }>["result"]);
      onError?(err: never);
    }
  ): () => void {
    let subscriptionId = undefined;
    let unsubscribed = false;

    const cleanup = () => {
      this.subscriptionMap?.delete(subscriptionId);
      if (subscriptionId) {
        this.transport.doRequest("subscriptionRemove", [subscriptionId]);
      }
    };

    this.transport.doRequest("subscriptionAdd", key).then((id) => {
      subscriptionId = id;
      if (unsubscribed) {
        cleanup();
      } else {
        this.subscriptionMap?.set(subscriptionId, options.onNext);
      }
    });

    return () => {
      unsubscribed = true;
      cleanup();
    };
  }
}
