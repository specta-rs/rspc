import { randomId, OperationType, Transport, RSPCError } from "@rspc/client";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrent } from "@tauri-apps/api/window";

export class TauriTransport implements Transport {
  private requestMap = new Map<string, (data: any) => void>();
  private listener?: Promise<UnlistenFn>;
  clientSubscriptionCallback?: (id: string, value: any) => void;

  constructor() {
    this.listener = listen("plugin:rspc:transport:resp", (event) => {
      const { id, result } = event.payload as any;
      if (result.type === "event") {
        if (this.clientSubscriptionCallback)
          this.clientSubscriptionCallback(id, result.data);
      } else if (result.type === "response") {
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: "response", result: result.data });
          this.requestMap.delete(id);
        }
      } else if (result.type === "error") {
        const { message, code } = result.data;
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: "error", message, code });
          this.requestMap.delete(id);
        }
      } else {
        console.error(`Received event of unknown method '${result.type}'`);
      }
    });
  }

  async doRequest(
    operation: OperationType,
    key: string,
    input: any
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

    await getCurrent().emit("plugin:rspc:transport", {
      id,
      method: operation,
      params: {
        path: key,
        input,
      },
    });

    const body = (await promise) as any;
    if (body.type === "error") {
      const { code, message } = body;
      throw new RSPCError(code, message);
    } else if (body.type === "response") {
      return body.result;
    } else {
      throw new Error(
        `RSPC Tauri doRequest received invalid body type '${body?.type}'`
      );
    }
  }
}
