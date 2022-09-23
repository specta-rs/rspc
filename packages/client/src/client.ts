import { OperationType, ProcedureKey, Procedures, RSPCError } from ".";
import { Transport } from "./transport";

export interface ClientTransformer {
  serialize(type: OperationType, key: ProcedureKey): ProcedureKey;
  deserialize(type: OperationType, key: ProcedureKey, data: any): any;
}
export interface ClientArgs {
  transport: Transport;
  transformer?: ClientTransformer;
  onError?: (err: RSPCError) => void | Promise<void>;
}

export function createClient<T extends Procedures>(
  args: ClientArgs
): Client<T> {
  return new Client(args);
}

export class Client<T extends Procedures> {
  private transport: Transport;
  private subscriptionMap = new Map<string, (data: any) => void>();
  private onError?: (err: RSPCError) => void | Promise<void>;

  constructor(args: ClientArgs) {
    this.transport = args.transport;
    this.transport.transformer = args.transformer;
    this.transport.clientSubscriptionCallback = (id, key, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
    this.onError = args.onError;
  }

  async query<K extends T["queries"]["key"]>(
    key: K
  ): Promise<Extract<T["queries"], { key: K }>["result"]> {
    try {
      return await this.transport.doRequest("query", key);
    } catch (err) {
      if (this.onError) {
        this.onError(err);
      } else {
        throw err;
      }
    }
  }

  async mutation<K extends T["mutations"]["key"][0]>(
    key: [K, Extract<T["mutations"], { key: [K, any] }>["key"][1]]
  ): Promise<Extract<T["mutations"], { key: K }>["result"]> {
    try {
      return await this.transport.doRequest("mutation", key);
    } catch (err) {
      if (this.onError) {
        this.onError(err);
      } else {
        throw err;
      }
    }
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
    try {
      let subscriptionId = undefined;
      let unsubscribed = false;

      const cleanup = () => {
        this.subscriptionMap?.delete(subscriptionId);
        if (subscriptionId) {
          this.transport.doRequest("subscriptionStop", [subscriptionId]);
        }
      };

      this.transport.doRequest("subscription", key).then((id) => {
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
    } catch (err) {
      if (this.onError) {
        this.onError(err);
      } else {
        throw err;
      }
    }
  }
}
