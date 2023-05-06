import { AlphaRSPCError, Link, RspcRequest } from "@rspc/client/v2";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

/**
 * Link for the rspc Tauri plugin
 */
export function tauriLink(): Link {
  const activeMap = new Map<
    string,
    {
      resolve: (result: any) => void;
      reject: (error: Error | AlphaRSPCError) => void;
    }
  >();
  const listener = listen("plugin:rspc:transport:resp", (event) => {
    const { id, result } = event.payload as any;
    if (activeMap.has(id)) {
      if (result.type === "event") {
        activeMap.get(id)?.resolve(result.data);
      } else if (result.type === "response") {
        activeMap.get(id)?.resolve(result.data);
        activeMap.delete(id);
      } else if (result.type === "error") {
        const { message, code } = result.data;
        activeMap.get(id)?.reject(new AlphaRSPCError(code, message));
        activeMap.delete(id);
      } else {
        console.error(`rspc: received event of unknown type '${result.type}'`);
      }
    } else {
      console.error(`rspc: received event for unknown id '${id}'`);
    }
  });

  const batch: RspcRequest[] = [];
  let batchQueued = false;
  const queueBatch = () => {
    if (!batchQueued) {
      batchQueued = true;
      setTimeout(() => {
        const currentBatch = [...batch];
        batch.splice(0, batch.length);
        batchQueued = false;

        (async () => {
          if (!listener) {
            await listener;
          }

          await appWindow.emit("plugin:rspc:transport", currentBatch);
        })();
      });
    }
  };

  return ({ op }) => {
    let finished = false;
    return {
      exec: async (resolve, reject) => {
        activeMap.set(op.id, {
          resolve,
          reject,
        });

        // @ts-expect-error // TODO: Fix this
        batch.push({
          id: op.id,
          method: op.type,
          params: {
            path: op.path,
            input: op.input,
          },
        });
        queueBatch();
      },
      abort() {
        if (finished) return;
        finished = true;

        const subscribeEventIdx = batch.findIndex((b) => b.id === op.id);
        if (subscribeEventIdx === -1) {
          if (op.type === "subscription") {
            // @ts-expect-error // TODO: Fix this
            batch.push({
              id: op.id,
              method: "subscriptionStop",
              params: null,
            });
            queueBatch();
          }
        } else {
          batch.splice(subscribeEventIdx, 1);
        }

        activeMap.delete(op.id);
      },
    };
  };
}
