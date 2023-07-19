import { RSPCError } from "../error";
import { Link } from "./link";
import { Request as RspcRequest, Response as RspcResponse } from "../bindings";
import { _internal_fireResponse } from "../internal";

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
 *
 * Note: This link applies request batching by default.
 */
// TODO: Deal with duplicate subscription id -> Retry -> Make the backend just give it a new ID in the response
// TODO: Reconnect all active subscriptions on connection restart
export function wsLink(opts: WsLinkOpts): Link {
  return _internal_wsLinkInternal(newWsManager(opts));
}

// TODO: Move into `@rspc/client/internal`
/**
 * @internal
 */
export function _internal_wsLinkInternal([activeMap, send]: ReturnType<
  typeof newWsManager
>): Link {
  let idCounter = 0; // TODO: Deal with integer overflow

  const batch: RspcRequest[] = []; // TODO: Change this to be `BatchedItem` and refactor
  let batchQueued = false;
  const queueBatch = () => {
    if (!batchQueued) {
      batchQueued = true;
      setTimeout(() => {
        let batches: RspcRequest[][];
        if (batch.length > 10) {
          batches = [];
          let batchIdx = 0;
          for (let i = 0; i < batch.length; i++) {
            if (i % 10 === 0) {
              batches.push([]);
              batchIdx++;
            }
            // @ts-expect-error
            batches[batchIdx - 1].push(batch[i]);
          }
        } else {
          batches = [batch];
        }

        send([...batch]);
        batch.splice(0, batch.length);
        batchQueued = false;
      });
    }
  };

  return ({ op: { context, ...op } }) => {
    // TODO: Get backend to send response if a subscription task crashes so we can unsubscribe from that subscription
    // TODO: If the current WebSocket is closed we should mark them all as finished because the tasks were killed on the server

    let finished = false;

    let id = idCounter++;
    return {
      exec: async (resolve, reject) => {
        activeMap.set(id, {
          oneshot: op.method !== "subscription",
          resolve,
          reject,
        });

        batch.push({
          id,
          ...op,
        });
        queueBatch();
      },
      abort() {
        if (finished) return;
        finished = true;

        const subscribeEventIdx = batch.findIndex(
          (b) =>
            b.method === "subscription" &&
            op.method === "subscription" &&
            b.id === id
        );
        if (subscribeEventIdx === -1) {
          if (op.method === "subscription") {
            batch.push({
              id,
              method: "subscriptionStop",
            });
            queueBatch();
          }
        } else {
          batch.splice(subscribeEventIdx, 1);
        }

        activeMap.delete(id);
      },
    };
  };
}

function newWsManager(opts: WsLinkOpts) {
  const WebSocket = opts.WebSocket || globalThis.WebSocket.bind(globalThis);
  const activeMap = new Map<
    number,
    {
      // Should delete after first response
      oneshot: boolean;
      resolve: (result: any) => void;
      reject: (error: Error | RSPCError) => void;
    }
  >();

  let ws: WebSocket;
  const attachEventListeners = () => {
    ws.addEventListener("message", (event) => {
      const results: RspcResponse[] = JSON.parse(event.data);
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
