// TODO: Redo this entire system when links are introduced
// TODO: Make this file work off Typescript types which are exported from Rust to ensure internal type-safety!
import { OperationType, RSPCError } from ".";

// TODO
export interface Transport {
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  doRequest(operation: OperationType, key: string, input: any): Promise<any>;
}

// TODO
export class FetchTransport implements Transport {
  private url: string;
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;
  private fetch: typeof globalThis.fetch;

  constructor(url: string, fetch?: typeof globalThis.fetch) {
    this.url = url;
    this.fetch = fetch || globalThis.fetch.bind(globalThis);
  }

  async doRequest(
    operation: OperationType,
    key: string,
    input: any
  ): Promise<any> {
    if (operation === "subscription" || operation === "subscriptionStop") {
      throw new Error(
        `Subscribing to '${key}' failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?`
      );
    }

    let method = "GET";
    let body = undefined as any;
    let headers = new Headers();

    const params = new URLSearchParams();
    if (operation === "query") {
      if (input !== undefined) {
        params.append("input", JSON.stringify(input));
      }
    } else if (operation === "mutation") {
      method = "POST";
      body = JSON.stringify(input || {});
      headers.set("Content-Type", "application/json");
    }
    const paramsStr = params.toString();
    const resp = await this.fetch(
      `${this.url}/${key}${paramsStr.length > 0 ? `?${paramsStr}` : ""}`,
      {
        method,
        body,
        headers,
      }
    );

    const respBody = await resp.json();
    const { type, data } = respBody.result;
    if (type === "error") {
      const { code, message } = data;
      throw new RSPCError(code, message);
    }
    return data;
  }
}

export const randomId = () => Math.random().toString(36).slice(2);

const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

export class WebsocketTransport implements Transport {
  private url: string;
  private ws: WebSocket;
  private requestMap = new Map<
    string,
    {
      op: unknown;
      cb: (data: any) => void;
    }
  >();
  clientSubscriptionCallback?: (id: string, value: any) => void;

  constructor(url: string) {
    this.url = url;
    this.ws = new WebSocket(url);
    this.attachEventListeners();
  }

  attachEventListeners() {
    // Resume all in-progress tasks
    for (const [_, item] of this.requestMap) {
      this.ws.send(JSON.stringify(item.op));
    }

    this.ws.addEventListener("message", (event) => {
      const { id, result } = JSON.parse(event.data);
      if (result.type === "event") {
        if (this.clientSubscriptionCallback)
          this.clientSubscriptionCallback(id, result.data);
      } else if (result.type === "response") {
        if (this.requestMap.has(id)) {
          this.requestMap
            .get(id)
            ?.cb({ type: "response", result: result.data });
          this.requestMap.delete(id);
        }
      } else if (result.type === "error") {
        const { message, code } = result.data;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.cb({ type: "error", message, code });
          this.requestMap.delete(id);
        }
      } else {
        console.error(`Received event of unknown type '${result.type}'`);
      }
    });

    this.ws.addEventListener("close", (event) => {
      this.reconnect();
    });
  }

  async reconnect(timeoutIndex = 0) {
    let timeout =
      (timeouts[timeoutIndex] ?? timeouts[timeouts.length - 1]) +
      (Math.floor(Math.random() * 5000 /* 5 Seconds */) + 1);

    setTimeout(() => {
      let ws = new WebSocket(this.url);
      new Promise(function (resolve, reject) {
        ws.addEventListener("open", () => resolve(null));
        ws.addEventListener("close", reject);
      })
        .then(() => {
          this.ws = ws;
          this.attachEventListeners();
        })
        .catch((err) => this.reconnect(timeoutIndex++));
    }, timeout);
  }

  async doRequest(
    operation: OperationType,
    key: string,
    input: any,
    opts?: {
      id?: string;
    }
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

    this.requestMap.set(id, {
      op: {
        id,
        method: operation,
        params: {
          path: key,
          input,
        },
      },
      // @ts-ignore
      cb: resolve,
    });

    this.ws.send(
      JSON.stringify({
        id,
        method: operation,
        params: {
          path: key,
          input,
        },
      })
    );

    const body = (await promise) as any;
    if (body.type === "error") {
      const { code, message } = body;
      throw new RSPCError(code, message);
    } else if (body.type === "response") {
      return body.result;
    } else {
      throw new Error(
        `RSPC Websocket doRequest received invalid body type '${body?.type}'`
      );
    }
  }
}

// TODO
export class NoOpTransport implements Transport {
  constructor() {}

  async doRequest(
    operation: OperationType,
    key: string,
    input: string
  ): Promise<any> {
    return new Promise(() => {});
  }
}
