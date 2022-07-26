import { Transport } from "./transport";

export type OperationKey = [string] | [string, /* args */ any];

export type OperationsDef = {
  queries: { key: OperationKey; result: any };
  mutations: { key: OperationKey; result: any };
  subscriptions: { key: OperationKey; result: any };
};

export interface ClientArgs {
  transport: Transport;
}

export function createClient<T extends OperationsDef>(
  args: ClientArgs
): Client<T> {
  return new Client(args);
}

export class Client<T extends OperationsDef> {
  private transport: Transport;

  constructor(args: ClientArgs) {
    this.transport = args.transport;
  }

  async query<K extends T["queries"]["key"]>(
    key: K
  ): Promise<Extract<T["queries"], { key: K }>["result"]> {
    return await this.transport.doRequest("query", key);
  }

  async mutation<K extends T["mutations"]["key"]>(
    key: K
  ): Promise<Extract<T["mutations"], { key: K }>["result"]> {
    return await this.transport.doRequest("mutation", key);
  }

  async addSubscription<K extends T["subscriptions"]["key"]>(
    key: K,
    options?: {
      onNext(msg: Extract<T["subscriptions"], { key: K }>["result"]);
      onError(err: never); // TODO: Error type??
    }
  ) {
    const id = await this.transport.subscribe(
      "subscriptionAdd",
      key,
      options?.onNext as any,
      options?.onError as any
    );
    console.log("SUBSCRIPTION ID", id);
  }

  // async removeSubscription<K extends T["subscriptions"]["key"]>(
  //   key: K,
  //   options?: {
  //     onNext(msg: Extract<T["subscriptions"], { key: K }>["result"]);
  //     onError(err: never); // TODO: Error type??
  //   }
  // ) {
  //   await this.transport.subscribe(
  //     "subscriptionAdd",
  //     key,
  //     options?.onNext as any,
  //     options?.onError as any
  //   );
  // }
}
