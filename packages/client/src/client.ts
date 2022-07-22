import { Transport } from "./transport";

export type OperationKey = [string] | [string, /* args */ any];

export type OperationsDef = {
  queries: { key: OperationKey; margs: any; arg: any; result: any };
  mutations: { key: OperationKey; margs: any; arg: any; result: any };
  subscriptions: { key: OperationKey; margs: any; arg: any; result: any };
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
    key: K,
    margs: Extract<T["queries"], { key: K }>["margs"]
  ): Promise<Extract<T["queries"], { key: K }>["result"]> {
    return await this.transport.doRequest("query", key, margs);
  }

  async mutation<K extends T["mutations"]["key"]>(
    key: K
  ): Promise<Extract<T["mutations"], { key: K }>["result"]> {
    return await this.transport.doRequest("mutation", key);
  }

  async subscription<K extends T["subscriptions"]["key"]>(
    key: K,
    options?: {
      onNext(msg: Extract<T["queries"], { key: K }>["result"]);
      onError(err: never); // TODO: Error type??
    }
  ) {
    // TODO: Handle unsubscribe
    this.transport.subscribe(
      "subscriptionAdd",
      key,
      options?.onNext as any,
      options?.onError as any
    );
  }
}
