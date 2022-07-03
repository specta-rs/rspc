import { Transport } from "./transport";

export type OperationsDef = {
  queries: { key: string; arg: any; result: any };
  mutations: { key: string; arg: any; result: any };
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
    arg?: Extract<T["queries"], { key: K }>["arg"]
  ): Promise<Extract<T["queries"], { key: K }>["result"]> {
    return await this.transport.doRequest(key, arg);
  }

  async mutation<K extends T["mutations"]["key"]>(
    key: K,
    arg?: Extract<T["mutations"], { key: K }>["arg"]
  ): Promise<Extract<T["mutations"], { key: K }>["result"]> {
    return await this.transport.doRequest(key, arg);
  }
}
