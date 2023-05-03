import { AlphaRSPCError } from "../error";
import { Link } from "./link";
import { Request as RspcRequest } from "../../bindings";

// TODO: keep this internal and don't export -> is used by `@rspc/react` so avoid that -> subscription management should be done by client.
export const randomId = () => Math.random().toString(36).slice(2);

const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

type WsLinkOpts = {
  url: string;
  /**
   * Add ponyfill for WebSocket
   */
  WebSocket?: typeof WebSocket;
};

/**
 * Websocket link for rspc
 */
export function wsLink(opts: WsLinkOpts): Link {
  const WebSocket = opts.WebSocket || globalThis.WebSocket.bind(globalThis);
  const activeMap = new Map<
    string,
    {
      resolve: (result: any) => void;
      reject: (error: Error | AlphaRSPCError) => void;
    }
  >();

  let ws: WebSocket;
  const attachEventListeners = () => {
    ws.addEventListener("message", (event) => {
      const { id, result } = JSON.parse(event.data);
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
          console.error(
            `rspc: received event of unknown type '${result.type}'`
          );
        }
      } else {
        console.error(`rspc: received event for unknown id '${id}'`);
      }
    });

    ws.addEventListener("close", (event) => {
      reconnectWs();
    });
  };

  const reconnectWs = (timeoutIndex = 0) => {
    let timeout =
      // @ts-expect-error // TODO: Fix this
      (timeouts[timeoutIndex] ?? timeouts[timeouts.length - 1]) +
      (Math.floor(Math.random() * 5000 /* 5 Seconds */) + 1);

    setTimeout(() => {
      let newWs = new WebSocket(opts.url);
      new Promise(function (resolve, reject) {
        newWs.addEventListener("open", () => resolve(null));
        newWs.addEventListener("close", reject);
      })
        .then(() => {
          ws = newWs;
          attachEventListeners();
        })
        .catch((err) => reconnectWs(timeoutIndex++));
    }, timeout);
  };

  const initWebsocket = () => {
    ws = new WebSocket(opts.url);
    attachEventListeners();
  };
  initWebsocket();

  const awaitWebsocketReady = async () => {
    if (ws.readyState == 0) {
      let resolve: () => void;
      const promise = new Promise((res) => {
        resolve = () => res(undefined);
      });
      ws.addEventListener("open", () => resolve());
      await promise;
    }
  };

  return ({ op }) => {
    const id = randomId();

    // TODO: Get backend to send response if a subscription task crashes so we can unsubscribe from that subscription
    // TODO: If the current WebSocket is closed we should mark them all as finished because the tasks were killed on the server

    let finished = false;
    return {
      exec: async (resolve, reject) => {
        await awaitWebsocketReady();

        activeMap.set(id, {
          resolve,
          reject,
        });

        ws.send(
          JSON.stringify({
            id,
            method: op.type,
            params: {
              path: op.path,
              input: op.input,
            },
          })
        );
      },
      abort() {
        if (finished) return;
        finished = true;

        activeMap.delete(id);

        awaitWebsocketReady().then(() => {
          const req: Extract<RspcRequest, { method: "subscriptionStop" }> = {
            jsonrpc: "2.0",
            id,
            method: "subscriptionStop",
            params: null,
          };

          ws.send(JSON.stringify(req));
        });
      },
    };
  };
}
