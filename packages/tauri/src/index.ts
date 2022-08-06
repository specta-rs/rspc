import {
  randomId,
  ClientTransformer,
  OperationKey,
  OperationType,
  Transport,
  RSPCError,
} from "@rspc/client";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

export class TauriTransport implements Transport {
  private requestMap = new Map<string, (data: any) => void>();
  private listener?: Promise<UnlistenFn>;
  transformer?: ClientTransformer;
  clientSubscriptionCallback?: (id: string, key: string, value: any) => void;

  constructor() {
    this.listener = listen("plugin:rspc:transport:resp", (event) => {
      const body = event.payload as any;
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
        console.error(`Received event of unknown method '${body.type}'`);
      }
    });
  }

  async doRequest(operation: OperationType, key: OperationKey): Promise<any> {
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
      operation,
      key: this.transformer?.serialize(operation, key) || key,
    });

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
        `RSPC Tauri doRequest received invalid body type '${body?.type}'`
      );
    }
  }
}
