import { ClientTransformer, ProcedureKey, OperationType, RSPCError } from ".";

// TODO: Make this file work off Typescript types which are exported from Rust to ensure internal type-safety!

export interface Transport {
  transformer?: ClientTransformer;
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  doRequest(operation: OperationType, key: ProcedureKey): Promise<any>;
}

export class FetchTransport implements Transport {
  private url: string;
  transformer?: ClientTransformer;
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  constructor(url: string) {
    this.url = url;
  }

  async doRequest(operation: OperationType, key: ProcedureKey): Promise<any> {
    if (operation === "subscription" || operation === "subscriptionStop") {
      throw new Error(
        `Subscribing to '${key[0]}' failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?`
      );
    }

    let method = "GET";
    let body = undefined as any;
    let headers = new Headers();

    const params = new URLSearchParams();
    key = this.transformer?.serialize(operation, key) || key;
    if (operation === "query") {
      if (key[1] !== undefined) {
        params.append("input", JSON.stringify(key[1]));
      }
    } else if (operation === "mutation") {
      method = "POST";
      body = JSON.stringify(key[1] || {});
      headers.set("Content-Type", "application/json");
    }
    const paramsStr = params.toString();
    const resp = await fetch(
      `${this.url}/${key[0]}${paramsStr.length > 0 ? `?${paramsStr}` : ""}`,
      {
        method,
        body,
        headers,
      }
    );

    const respBody = (await resp.json())[0]; // TODO: Batching
    const { type, result } = respBody;
    if (type === "error") {
      const { status_code, message } = respBody;
      throw new RSPCError(status_code, message);
    }
    return this.transformer?.deserialize(operation, key, result) || result;
  }
}

export const randomId = () => Math.random().toString(36).slice(2);

const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

export class WebsocketTransport implements Transport {
  private url: string;
  private ws: WebSocket;
  private requestMap = new Map<string, (data: any) => void>();
  transformer?: ClientTransformer;
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  constructor(url: string) {
    this.url = url;
    this.ws = new WebSocket(url);
    this.attachEventListeners();
  }

  attachEventListeners() {
    this.ws.addEventListener("message", (event) => {
      const body = JSON.parse(event.data);
      if (body.type === "event") {
        const { id, key, result } = body;
        this.clientSubscriptionCallback(id, key, result);
      } else if (body.type === "response") {
        const { id, result } = body;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: "response", result });
          this.requestMap.delete(id);
        }
      } else if (body.type === "error") {
        const { id, message, status_code } = body;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: "error", message, status_code });
          this.requestMap.delete(id);
        }
      } else {
        console.error(`Received event of unknown type '${body.type}'`);
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

  async doRequest(operation: OperationType, key: ProcedureKey): Promise<any> {
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

    const key2 = this.transformer?.serialize(operation, key) || key;
    this.ws.send(
      JSON.stringify({
        id,
        method: operation,
        params: {
          path: key2[0],
          input: key2[1],
        },
      })
    );

    const body = (await promise) as any;
    if (body.type === "error") {
      const { status_code, message } = body;
      throw new RSPCError(status_code, message);
    } else if (body.type === "response") {
      return (
        this.transformer?.deserialize(operation, key, body.result) ||
        body.result
      );
    } else {
      throw new Error(
        `RSPC Websocket doRequest received invalid body type '${body?.type}'`
      );
    }
  }
}
