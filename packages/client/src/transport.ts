// TODO: Make this file work off Typescript types which are exported from Rust to ensure internal type-safety!

export type OperationType =
  | "query"
  | "mutation"
  | "subscriptionAdd"
  | "subscriptionRemove";

export interface Transport {
  doRequest(operation: OperationType, key: string, arg: any): Promise<any>;
}

export class FetchTransport implements Transport {
  private url: string;

  constructor(url: string) {
    this.url = url;
  }

  async doRequest(
    operation: OperationType,
    key: string,
    arg: any
  ): Promise<any> {
    let url = `${this.url}/${key}`;
    let method = "GET";
    let body = undefined as any;
    let headers = new Headers();

    if (operation === "query") {
      url += `?batch=1&input=${encodeURIComponent(arg || "{}")}`;
    } else if (operation === "mutation") {
      url += `?batch=1`;
      method = "POST";
      body = arg || {};
      headers.set("Content-Type", "application/json");
    }

    const resp = await fetch(url, {
      method,
      body: body ? JSON.stringify(body) : undefined,
      headers,
    });
    // TODO: Error handling
    return (await resp.json())[0].result.data;
  }
}

const randomId = () => Math.random().toString(36).slice(2);

export class WebsocketTransport implements Transport {
  private url: string;
  private ws: WebSocket;
  private requestMap = new Map<string, (data: any) => void>();

  constructor(url: string) {
    this.url = url;
    this.ws = new WebSocket(url);

    this.ws.addEventListener("message", (event) => {
      const { id, result } = JSON.parse(event.data);

      if (this.requestMap.has(id)) {
        this.requestMap.get(id)?.(result);
        this.requestMap.delete(id);
      }
    });
  }

  async doRequest(
    operation: OperationType,
    key: string,
    arg: any
  ): Promise<any> {
    if (this.ws.readyState == 0) {
      let resolve: () => void;
      const promise = new Promise((res) => {
        resolve = () => res(undefined);
      });
      // @ts-ignore
      this.ws.addEventListener("open", resolve);
      await promise;
    }

    const id = randomId();
    let resolve: (data: any) => void;
    const promise = new Promise((res) => {
      resolve = res;
    });

    // @ts-ignore
    this.requestMap.set(id, resolve);

    this.ws.send(
      JSON.stringify({
        id,
        method: operation,
        operation: key,
        arg: arg || {},
      })
    );

    return await promise;
  }
}
