import {
  Operation,
  ProcedureType,
  TRPCClientOutgoingMessage,
  UnsubscribeFn,
  TRPCRequestMessage,
  TRPCWebSocketClient,
  ProceduresDef,
  TRPCLink,
  wsLink,
} from "@rspc/client";
import { listen } from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

type TCallbacks = any; // TODO

export function tauriLink<
  TProcedures extends ProceduresDef
>(): TRPCLink<TProcedures> {
  return wsLink<TProcedures>({
    client: createTauriClient(),
  });
}

export function createTauriClient(): TRPCWebSocketClient {
  /**
   * outgoing messages buffer whilst not open
   */
  let outgoing: TRPCClientOutgoingMessage[] = [];
  /**
   * pending outgoing requests that are awaiting callback
   */
  type TRequest = {
    /**
     * Reference to the WebSocket instance this request was made to
     */
    ws: WebSocket;
    type: ProcedureType;
    callbacks: TCallbacks;
    op: Operation;
  };
  const pendingRequests: Record<number | string, TRequest> =
    Object.create(null);
  let dispatchTimer: ReturnType<typeof setTimeout> | number | null = null;
  let state: "open" | "closed" = "open";

  function dispatch() {
    if (state !== "open" || dispatchTimer) {
      return;
    }
    dispatchTimer = setTimeout(() => {
      dispatchTimer = null;

      if (outgoing.length === 0) {
        return;
      }

      if (outgoing.length === 1) {
        // single send
        appWindow.emit("plugin:rspc:transport", JSON.stringify(outgoing.pop()));
      } else {
        // batch send
        appWindow.emit("plugin:rspc:transport", JSON.stringify(outgoing));
      }
      // clear
      outgoing = [];
    });
  }

  listen("plugin:rspc:transport:resp", (event) => {
    const data = event.payload as any;
    if ("method" in data) {
    } else {
      const req = data.id !== null && pendingRequests[data.id];
      if (!req) {
        // do something?
        return;
      }
      req.callbacks.next?.(data);
      if ("result" in data && data.result.type === "stopped") {
        req.callbacks.complete();
      }
    }
  }).then(() => {
    state = "open";
    dispatch();
  });

  function request(op: Operation, callbacks: TCallbacks): UnsubscribeFn {
    const { type, input, path, id } = op;
    const envelope: TRPCRequestMessage = {
      id,
      method: type,
      params: {
        input,
        path,
      },
    };
    pendingRequests[id] = {
      ws: undefined as any, // TODO: Remove this field
      type,
      callbacks,
      op,
    };
    // enqueue message
    outgoing.push(envelope);
    dispatch();
    return () => {
      const callbacks = pendingRequests[id]?.callbacks;
      delete pendingRequests[id];
      outgoing = outgoing.filter((msg) => msg.id !== id);
      callbacks?.complete?.();
      if (op.type === "subscription") {
        outgoing.push({
          id,
          method: "subscriptionStop",
        });
        dispatch();
      }
    };
  }

  return {
    close: () => {
      state = "closed";
      // TODO: Close all open subscriptions
      //   closeIfNoPending(activeConnection);
      // TODO
    },
    request,
  };
}
