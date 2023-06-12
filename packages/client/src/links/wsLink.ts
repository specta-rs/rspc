// import { RSPCError } from "../error";
// import { Link, Operation } from "./link";
// import { Request as RspcRequest, Response as RspcResponse } from "../bindings";
// import { MapKey, deduplicatorWs, toMapId } from "../internal";

// const timeouts = [1000, 2000, 5000, 10000]; // In milliseconds

// type WsLinkOpts = {
//   url: string;
//   /**
//    * Add ponyfill for WebSocket
//    */
//   WebSocket?: typeof WebSocket;
// };

// function newWsManager(opts: WsLinkOpts) {
//   const WebSocket = opts.WebSocket || globalThis.WebSocket.bind(globalThis);
//   const d = deduplicatorWs({});

//   // TODO: Max size of batch, if greater than x items send in multiple batches -> For both WS and HTTP link

//   let ws: WebSocket;
//   const attachEventListeners = () => {
//     ws.addEventListener("message", (event) => {
//       const results: RspcResponse[] = JSON.parse(event.data);
//       for (const result of results) {
//         const id = toMapId(result);
//         const item = activeMap.get(id);

//         console.log(id, item, activeMap.keys()); // TODO

//         if (!item) {
//           console.error(`rspc: received event for unknown [${id.join(",")}]`);
//           return;
//         }

//         if (result.result.type === "value") {
//           item.resolve(result.result.value);
//         } else if (result.result.type === "error") {
//           item.reject(
//             new RSPCError(result.result.value.code, result.result.value.message)
//           );
//         } else {
//           console.error(
//             `rspc: received response of unknown type for [${id.join(",")}]`
//           );
//         }

//         item.resolve(result.result);
//         if ("path" in event) activeMap.delete(id);
//       }
//     });

//     ws.addEventListener("close", (event) => {
//       reconnectWs();
//     });
//   };

//   const reconnectWs = (timeoutIndex = 0) => {
//     let timeout =
//       // @ts-expect-error // TODO: Fix this
//       (timeouts[timeoutIndex] ?? timeouts[timeouts.length - 1]) +
//       (Math.floor(Math.random() * 5000 /* 5 Seconds */) + 1);

//     setTimeout(() => {
//       let newWs = new WebSocket(opts.url);
//       new Promise(function (resolve, reject) {
//         newWs.addEventListener("open", () => resolve(null));
//         newWs.addEventListener("close", reject);
//       })
//         .then(() => {
//           ws = newWs;
//           attachEventListeners();
//         })
//         .catch((err) => reconnectWs(timeoutIndex++));
//     }, timeout);
//   };

//   const initWebsocket = () => {
//     ws = new WebSocket(opts.url);
//     attachEventListeners();
//   };
//   initWebsocket();

//   const awaitWebsocketReady = async () => {
//     if (ws.readyState == 0) {
//       let resolve: () => void;
//       const promise = new Promise((res) => {
//         resolve = () => res(undefined);
//       });
//       ws.addEventListener("open", () => resolve());
//       await promise;
//     }
//   };

//   return [
//     activeMap,
//     (data: RspcRequest | RspcRequest[]) =>
//       awaitWebsocketReady().then(() => ws.send(JSON.stringify(data))),
//   ] as const;
// }

// /**
//  * Websocket link for rspc
//  *
//  * Note: This link applies request batching by default.
//  */
// export function wsBatchLink(opts: WsLinkOpts): Link {
//   const [activeMap, send] = newWsManager(opts);

//   const batch: RspcRequest[] = [];
//   let batchQueued = false;
//   const queueBatch = () => {
//     if (!batchQueued) {
//       batchQueued = true;
//       setTimeout(() => {
//         send([...batch]);
//         batch.splice(0, batch.length);
//         batchQueued = false;
//       });
//     }
//   };

//   return ({ op }) => {
//     // TODO: Get backend to send response if a subscription task crashes so we can unsubscribe from that subscription
//     // TODO: If the current WebSocket is closed we should mark them all as finished because the tasks were killed on the server

//     let finished = false;
//     return {
//       exec: async (resolve, reject) => {
//         activeMap.set(toMapId(op), {
//           resolve,
//           reject,
//         });

//         batch.push(op);
//         queueBatch();
//       },
//       abort() {
//         if (finished) return;
//         finished = true;

//         const subscribeEventIdx = batch.findIndex(
//           (b) =>
//             b.method === "subscription" &&
//             op.method === "subscription" &&
//             b.id === op.id
//         );
//         if (subscribeEventIdx === -1) {
//           if (op.method === "subscription") {
//             batch.push({
//               id: op.id,
//               method: "subscriptionStop",
//             });
//             queueBatch();
//           }
//         } else {
//           batch.splice(subscribeEventIdx, 1);
//         }

//         activeMap.delete(toMapId(op));
//       },
//     };
//   };
// }

export {}; // TODO
