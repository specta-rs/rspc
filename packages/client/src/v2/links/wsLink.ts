import { AlphaRSPCError } from "../error";
import { Link, Operation } from "./link";
import { Request as RspcRequest } from "../../bindings";

const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

type WsLinkOpts = {
  url: string;
  /**
   * Add ponyfill for WebSocket
   */
  WebSocket?: typeof WebSocket;
};

function newWsManager(opts: WsLinkOpts) {
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

  return [
    activeMap,
    (data: RspcRequest | RspcRequest[]) =>
      awaitWebsocketReady().then(() => ws.send(JSON.stringify(data))),
  ] as const;
}

/**
 * Websocket link for rspc
 */
export function wsLink(opts: WsLinkOpts): Link {
  const [activeMap, send] = newWsManager(opts);

  return ({ op }) => {
    // TODO: Get backend to send response if a subscription task crashes so we can unsubscribe from that subscription
    // TODO: If the current WebSocket is closed we should mark them all as finished because the tasks were killed on the server

    let finished = false;
    return {
      exec: async (resolve, reject) => {
        activeMap.set(op.id, {
          resolve,
          reject,
        });

        send({
          id: op.id,
          // @ts-expect-error // TODO: Fix this
          method: op.type,
          params: {
            path: op.path,
            input: op.input,
          },
        });
      },
      abort() {
        if (finished) return;
        finished = true;

        // TODO: We should probs still use dataloader internally to deal with create/delete events due to React strict mode.
        activeMap.delete(op.id);
        send({
          id: op.id,
          // @ts-expect-error // TODO: Fix this
          method: "subscriptionStop",
          // @ts-expect-error // TODO: Fix this
          params: null,
        });
      },
    };
  };
}

/**
 * Wrapper around wsLink that applies request batching.
 */
// TODO: Ability to use context to skip batching on certain operations
export function wsBatchLink(opts: WsLinkOpts): Link {
  const [activeMap, send] = newWsManager(opts);

  const batch: RspcRequest[] = [];
  let batchQueued = false;
  const queueBatch = () => {
    if (!batchQueued) {
      batchQueued = true;
      setTimeout(() => {
        send([...batch]);
        batch.splice(0, batch.length);
        batchQueued = false;
      });
    }
  };

  return ({ op }) => {
    // TODO: Get backend to send response if a subscription task crashes so we can unsubscribe from that subscription
    // TODO: If the current WebSocket is closed we should mark them all as finished because the tasks were killed on the server

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
