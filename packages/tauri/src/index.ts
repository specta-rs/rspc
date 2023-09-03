import {
  Link,
  _internal_wsLinkInternal,
  _internal_fireResponse,
  ProceduresDef,
} from "@rspc/client";
import * as rspc from "@rspc/client";
import { appWindow } from "@tauri-apps/api/window";
import { events } from "./types";

/**
 * Link for the rspc Tauri plugin
 */
export function tauriLink<P extends ProceduresDef>(): Link<P> {
  return _internal_wsLinkInternal(newWsManager());
}

function newWsManager<P extends ProceduresDef>() {
  const activeMap = new Map<
    number,
    {
      oneshot: boolean;
      resolve: (result: P[keyof ProceduresDef]["result"]) => void;
      reject: (error: P[keyof ProceduresDef]["error"] | rspc.Error) => void;
    }
  >();

  const listener = events.transportResp.listen((event) => {
    const results = event.payload;

    for (const result of results) {
      const item = activeMap.get(result.id);

      if (!item) {
        console.error(
          `rspc: received event with id '${result.id}' for unknown`
        );
        return;
      }

      _internal_fireResponse<P>(result, {
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
    (data: rspc.Request | rspc.Request[]) =>
      listener.then(() => events.msg(appWindow).emit(data)),
  ] as const;
}
