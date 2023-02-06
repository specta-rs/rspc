import { fullObservable, observable, Observer, UnsubscribeFn } from ".";
import {
  Link,
  OperationContext,
  ProceduresDef,
  RSPCError,
  transformResult,
} from "..";

// TODO: Remove all these type aliases or assign them to something useful
type TRPCClientIncomingMessage = any;

type TRPCClientIncomingRequest = any;
export type TRPCClientOutgoingMessage = any;
export type TRPCRequestMessage = any;
export type TRPCResponseMessage<A = unknown, B = unknown> = any;
type inferRouterError<A> = any;
export type ProcedureType = any;

type WSCallbackResult<
  TProcedures extends ProceduresDef,
  TOutput
> = TRPCResponseMessage<TOutput, inferRouterError<TProcedures>>;

export type WSCallbackObserver<
  TProcedures extends ProceduresDef,
  TOutput
> = Observer<WSCallbackResult<TProcedures, TOutput>, RSPCError>;

/**
 * @internal
 */
export type LegacyOperation<TInput = unknown> = {
  id: number;
  type: "query" | "mutation" | "subscription";
  input: TInput;
  path: string;
  context: OperationContext;
};

export const retryDelay = (attemptIndex: number) =>
  attemptIndex === 0 ? 0 : Math.min(1000 * 2 ** attemptIndex, 30000);

export interface WebSocketClientOptions {
  url: string;
  WebSocket?: typeof WebSocket;
  retryDelayMs?: typeof retryDelay;
  onOpen?: () => void;
  onClose?: (cause?: { code?: number }) => void;
}

export type TCallbacks = WSCallbackObserver<ProceduresDef, unknown>;

export function createWSClient(opts: WebSocketClientOptions) {
  const {
    url,
    WebSocket: WebSocketImpl = WebSocket,
    retryDelayMs: retryDelayFn = retryDelay,
    onOpen,
    onClose,
  } = opts;
  /* istanbul ignore next */
  if (!WebSocketImpl) {
    throw new Error(
      "No WebSocket implementation found - you probably don't want to use this on the server, but if you do you need to pass a `WebSocket`-ponyfill"
    );
  }
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
    op: LegacyOperation;
  };
  const pendingRequests: Record<number | string, TRequest> =
    Object.create(null);
  let connectAttempt = 0;
  let dispatchTimer: ReturnType<typeof setTimeout> | number | null = null;
  let connectTimer: ReturnType<typeof setTimeout> | number | null = null;
  let activeConnection = createWS();
  let state: "open" | "connecting" | "closed" = "connecting";
  /**
   * tries to send the list of messages
   */
  function dispatch() {
    if (state !== "open" || dispatchTimer) {
      return;
    }
    dispatchTimer = setTimeout(() => {
      dispatchTimer = null;

      if (outgoing.length === 1) {
        // single send
        activeConnection.send(JSON.stringify(outgoing.pop()));
      } else {
        // batch send
        activeConnection.send(JSON.stringify(outgoing));
      }
      // clear
      outgoing = [];
    });
  }
  function tryReconnect() {
    if (connectTimer || state === "closed") {
      return;
    }
    const timeout = retryDelayFn(connectAttempt++);
    reconnectInMs(timeout);
  }
  function reconnect() {
    state = "connecting";
    const oldConnection = activeConnection;
    activeConnection = createWS();
    closeIfNoPending(oldConnection);
  }
  function reconnectInMs(ms: number) {
    if (connectTimer) {
      return;
    }
    state = "connecting";
    connectTimer = setTimeout(reconnect, ms);
  }

  function closeIfNoPending(conn: WebSocket) {
    // disconnect as soon as there are are no pending result
    const hasPendingRequests = Object.values(pendingRequests).some(
      (p) => p.ws === conn
    );
    if (!hasPendingRequests) {
      conn.close();
    }
  }

  function resumeSubscriptionOnReconnect(req: TRequest) {
    if (outgoing.some((r) => r.id === req.op.id)) {
      return;
    }
    request(req.op, req.callbacks);
  }

  function createWS() {
    const conn = new WebSocketImpl(url);
    clearTimeout(connectTimer as any);
    connectTimer = null;

    conn.addEventListener("open", () => {
      /* istanbul ignore next */
      if (conn !== activeConnection) {
        return;
      }
      connectAttempt = 0;
      state = "open";
      onOpen?.();
      dispatch();
    });
    conn.addEventListener("error", () => {
      if (conn === activeConnection) {
        tryReconnect();
      }
    });
    const handleIncomingRequest = (req: TRPCClientIncomingRequest) => {
      if (req.method === "reconnect" && conn === activeConnection) {
        if (state === "open") {
          onClose?.();
        }
        reconnect();
        // notify subscribers
        for (const pendingReq of Object.values(pendingRequests)) {
          if (pendingReq.type === "subscription") {
            resumeSubscriptionOnReconnect(pendingReq);
          }
        }
      }
    };
    const handleIncomingResponse = (data: TRPCResponseMessage) => {
      const req = data.id !== null && pendingRequests[data.id];
      if (!req) {
        // do something?
        return;
      }

      req.callbacks.next?.(data);
      if (req.ws !== activeConnection && conn === activeConnection) {
        const oldWs = req.ws;
        // gracefully replace old connection with this
        req.ws = activeConnection;
        closeIfNoPending(oldWs);
      }

      if (
        "result" in data &&
        data.result.type === "stopped" &&
        conn === activeConnection
      ) {
        req.callbacks.complete();
      }
    };
    conn.addEventListener("message", ({ data }) => {
      const msg = JSON.parse(data) as TRPCClientIncomingMessage;

      if ("method" in msg) {
        handleIncomingRequest(msg);
      } else {
        handleIncomingResponse(msg);
      }
      if (conn !== activeConnection || state === "closed") {
        // when receiving a message, we close old connection that has no pending requests
        closeIfNoPending(conn);
      }
    });

    conn.addEventListener("close", ({ code }) => {
      if (state === "open") {
        onClose?.({ code });
      }

      if (activeConnection === conn) {
        // connection might have been replaced already
        tryReconnect();
      }

      for (const [key, req] of Object.entries(pendingRequests)) {
        if (req.ws !== conn) {
          continue;
        }
        req.callbacks.error?.(
          RSPCError.from(
            new TRPCWebSocketClosedError("WebSocket closed prematurely")
          )
        );
        if (req.type !== "subscription") {
          delete pendingRequests[key];
          req.callbacks.complete?.();
        } else if (state !== "closed") {
          // request restart of sub with next connection
          resumeSubscriptionOnReconnect(req);
        }
      }
    });
    return conn;
  }

  function request(op: LegacyOperation, callbacks: TCallbacks): UnsubscribeFn {
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
      ws: activeConnection,
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
      onClose?.();
      closeIfNoPending(activeConnection);
      clearTimeout(connectTimer as any);
      connectTimer = null;
    },
    request,
    getConnection() {
      return activeConnection;
    },
  };
}

export type TRPCWebSocketClient = {
  close(): void;
  request(op: LegacyOperation, callbacks: TCallbacks): UnsubscribeFn;
};

export interface WebSocketLinkOptions {
  client: TRPCWebSocketClient;
}
class TRPCWebSocketClosedError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TRPCWebSocketClosedError";
    Object.setPrototypeOf(this, TRPCWebSocketClosedError.prototype);
  }
}

class TRPCSubscriptionEndedError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TRPCSubscriptionEndedError";
    Object.setPrototypeOf(this, TRPCSubscriptionEndedError.prototype);
  }
}

// TODO: Redo `WebSocketLinkOptions` and clean up everything
export function wsLink<T extends ProceduresDef>(
  opts: WebSocketLinkOptions
): Link<T, T, { terminatedLink: true }> {
  const { client } = opts;

  return (op) => {
    // TODO
    return fullObservable(
      observable((observer) => {
        const { type, path, id, context, input } = op.op;

        let isDone = false;
        const unsub = client.request(
          { type, path, input, id, context },
          {
            error(err) {
              isDone = true;
              observer.error(err as RSPCError);
              unsub();
            },
            complete() {
              if (!isDone) {
                isDone = true;
                observer.error(
                  RSPCError.from(
                    new TRPCSubscriptionEndedError(
                      "Operation ended prematurely"
                    )
                  )
                );
              } else {
                observer.complete();
              }
            },
            next(message) {
              const transformed = transformResult(message);

              if (!transformed.ok) {
                const error = RSPCError.from(transformed.error);
                // TODO: `onError`
                // runtime.onError?.({
                //   error,
                //   path,
                //   input,
                //   ctx: context,
                //   type: type,
                // });
                observer.error(error);
                return;
              }
              observer.next({
                result: transformed.result,
              });

              if (op.op.type !== "subscription") {
                // if it isn't a subscription we don't care about next response

                isDone = true;
                unsub();
                observer.complete();
              }
            },
          }
        );
        return () => {
          isDone = true;
          unsub();
        };
      })
    );
  };
}
