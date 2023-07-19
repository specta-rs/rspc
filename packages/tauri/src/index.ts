import {
  Link,
  Request as RspcRequest,
  Response as RspcResponse,
  RSPCError,
  _internal_wsLinkInternal,
  _internal_fireResponse,
} from "@rspc/client";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

/**
 * Link for the rspc Tauri plugin
 */
export function tauriLink(): Link {
  return _internal_wsLinkInternal(newWsManager());
}

function newWsManager() {
  const activeMap = new Map<
    number,
    {
      oneshot: boolean;
      resolve: (result: any) => void;
      reject: (error: Error | RSPCError) => void;
    }
  >();

  const listener = listen("plugin:rspc:transport:resp", (event) => {
    const results: RspcResponse[] = JSON.parse(event.payload as any);

    for (const result of results) {
      const item = activeMap.get(result.id);

      if (!item) {
        console.error(
          `rspc: received event with id '${result.id}' for unknown`
        );
        return;
      }

      _internal_fireResponse(result, {
        resolve: item.resolve,
        reject: item.reject,
      });
      if (
        (item.oneshot && result.type === "value") ||
        result.type === "complete"
      )
        activeMap.delete(result.id);
    }
  });

  return [
    activeMap,
    (data: RspcRequest | RspcRequest[]) =>
      listener.then(() =>
        appWindow.emit("plugin:rspc:transport", JSON.stringify(data))
      ),
  ] as const;
}
