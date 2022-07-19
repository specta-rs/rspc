// @ts-ignore // TODO: Fix this
import { invoke } from "@tauri-apps/api";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

// TODO: Make this file work off Typescript types which are exported from Rust to ensure internal type-safety!

export type OperationType =
  | "query"
  | "mutation"
  | "subscriptionAdd"
  | "subscriptionRemove";

export interface Transport {
  doRequest(operation: OperationType, key: string, arg: any): Promise<any>;
  subscribe(
    operation: OperationType,
    key: string,
    arg: any,
    onMessage: (msg: any) => void,
    onError: (msg: any) => void
  ): Promise<void>;
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
      url += `?batch=1&input=${encodeURIComponent(JSON.stringify(arg || {}))}`;
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

  async subscribe(
    operation: OperationType,
    key: string,
    arg: any,
    onMessage: (msg: any) => void,
    onError: (msg: any) => void
  ): Promise<void> {
    console.error(
      `Subscribing to '{}' failed as the HTTP transport does not support subscriptions. Maybe try using Websockets?`
    );
  }
}

const randomId = () => Math.random().toString(36).slice(2);

export class WebsocketTransport implements Transport {
  private url: string;
  private ws: WebSocket;
  private requestMap = new Map<string, (data: any) => void>();
  private subscriptionMap = new Map<string, Set<(data: any) => void>>();

  constructor(url: string) {
    this.url = url;
    this.ws = new WebSocket(url);

    this.ws.addEventListener("message", (event) => {
      const body = JSON.parse(event.data);
      if (body.type === "event") {
        const { key, result } = body;
        this.subscriptionMap.get(key)?.forEach((func) => {
          func(result);
        });
      } else if (body.type === "response") {
        const { id, result } = body;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.(result);
          this.requestMap.delete(id);
        }
      } else {
        console.error(`Received event of unknown type '${body.type}'`);
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

  async subscribe(
    operation: OperationType,
    key: string,
    arg: any,
    onMessage?: (msg: any) => void,
    onError?: (msg: any) => void
  ): Promise<void> {
    if (this.ws.readyState == 0) {
      let resolve: () => void;
      const promise = new Promise((res) => {
        resolve = () => res(undefined);
      });
      // @ts-ignore
      this.ws.addEventListener("open", resolve);
      await promise;
    }

    if (!this.subscriptionMap.has(key)) {
      this.subscriptionMap.set(key, new Set());
    }

    if (onMessage) {
      this.subscriptionMap.get(key)?.add(onMessage);
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

    await promise;
  }
}

export class TauriTransport implements Transport {
  private requestMap = new Map<string, (data: any) => void>();
  private subscriptionMap = new Map<string, Set<(data: any) => void>>();
  private listener?: Promise<UnlistenFn>;

  constructor() {
    this.listener = listen("plugin:rspc:transport:resp", (event) => {
      const body = event.payload as any;
      if (body.type === "event") {
        // const { key, result } = body;
        // this.subscriptionMap.get(key)?.forEach((func) => {
        //   func(result);
        // });
      } else if (body.type === "response") {
        const { id, kind, result } = body;
        if (kind === "success") {
          if (this.requestMap.has(id)) {
            this.requestMap.get(id)?.(result);
            this.requestMap.delete(id);
          } else {
            console.error(`Missing handler for request with id '${id}'`);
          }
        } else {
          console.error(`Received event of unknown kind '${kind}'`);
        }
      } else {
        console.error(`Received event of unknown method '${body.type}'`);
      }
    });
  }

  async doRequest(
    operation: OperationType,
    key: string,
    arg: any
  ): Promise<any> {
    if (!this.listener) {
      await this.listener;
    }

    const id = randomId();
    let resolve: (data: any) => void;
    const promise = new Promise((res) => {
      resolve = res;
    });

    // @ts-ignore
    this.requestMap.set(id, resolve);

    await appWindow.emit("plugin:rspc:transport", {
      id,
      method: operation,
      operation: key,
      arg,
    });

    return await promise;
  }

  async subscribe(
    operation: OperationType,
    key: string,
    arg: any,
    onMessage: (msg: any) => void,
    onError: (msg: any) => void
  ): Promise<void> {
    // TODO
  }
}
