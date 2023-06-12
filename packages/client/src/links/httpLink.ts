import { ValueOrError } from "..";
import { RSPCError } from "../error";
import { BatchedItem, fireResponse } from "../internal";
import { Link, Operation } from "./link";

type HTTPHeaders = Record<string, string | string[] | undefined>;

type HttpLinkOpts = {
  url: string;
  /**
   * Add ponyfill for fetch
   */
  fetch?: typeof fetch;
  /**
   * Add ponyfill for AbortController
   */
  AbortController?: typeof AbortController | null;
} & (
  | {
      /**
       * Headers to be set on outgoing requests or a callback that of said headers
       */
      headers?: HTTPHeaders | ((opts: { op: Operation }) => HTTPHeaders);
    }
  | {
      /**
       * Headers to be set on outgoing requests or a callback that of said headers
       */
      headers?: HTTPHeaders | ((opts: { ops: Operation[] }) => HTTPHeaders);
      /**
       * Batch multiple rspc queries into a single HTTP request
       *
       * NOTE: This is great for performance but may be problematic if your using HTTP caching.
       */
      batch:
        | true
        | {
            maxBatchSize?: number; // TODO: Make this work
            shouldBatch?: (op: Operation) => boolean;
          };
    }
);

/**
 * HTTP Fetch link for rspc
 */
export function httpLink(opts: HttpLinkOpts): Link {
  const fetchFn = opts.fetch || globalThis.fetch.bind(globalThis);
  const abortController =
    opts.AbortController || globalThis.AbortController.bind(globalThis);

  if (!opts.url.endsWith("/")) opts.url += "/";

  const activeReqs = new Map<string, BatchedItem[]>(); // TODO: Deduplicate fetches by queryKey hash

  let dispatch: (op: BatchedItem) => void = (item: BatchedItem) => {
    const [url, init] = requestParams(opts, item);
    doFetch<ValueOrError>(fetchFn, opts.url + url, init).then((body) => {
      if (body === undefined) {
        return;
      }

      if (body instanceof RSPCError) {
        item.reject(body);

        return;
      }

      if (item.abort.signal?.aborted) {
        return;
      }

      fireResponse(body, item);
    });
  };

  if ("batch" in opts) {
    const doFetchBatched = async (batch: BatchedItem[]) => {
      const body = await doFetch<ValueOrError[]>(fetchFn, opts.url + "_batch", {
        method: "POST",
        headers: generateHeaders(opts, { ops: batch.map((b) => b.op) }),
        body: JSON.stringify(batch.map((b) => b.op)),
        // We don't handle the abort signal for a batch so a single req doesn't kill entire batch.
      });
      if (body === undefined) {
        return;
      }

      if (body instanceof RSPCError) {
        for (const item of batch) {
          item.reject(body);
        }

        return;
      }

      if (body.length !== batch.length) {
        // TODO: Send proper resp error to every item in batch
        console.error("rspc: batch response length mismatch!");
        for (const item of batch) {
          item?.reject(new RSPCError(500, "batch response length mismatch!"));
        }
        return;
      }

      for (const [i, item] of body.entries()) {
        const batchItem = batch[i]!;

        if (batchItem.abort.signal?.aborted) {
          continue;
        }

        fireResponse(item, batchItem);
      }
    };

    const batch: BatchedItem[] = [];
    let batchQueued = false;
    dispatch = (op: BatchedItem) => {
      if (
        (typeof opts.batch === "boolean" && opts.batch) ||
        (typeof opts.batch === "object" &&
          (opts.batch.shouldBatch?.(op.op) || true))
      ) {
        batch.push(op);

        if (!batchQueued) {
          batchQueued = true;
          setTimeout(() => {
            doFetchBatched([...batch]); // TODO
            batch.splice(0, batch.length);
            batchQueued = false;
          });
        }
      } else {
        dispatch(op);
      }
    };
  }

  return ({ op }) => {
    const abort = new abortController();
    return {
      exec: async (resolve, reject) => {
        if (op.method === "subscription") {
          reject(
            new Error(
              `Subscribing to '${op.path}' failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?`
            )
          );
          return;
        }
        // else if (op.method === "subscriptionStop") {
        //   reject(
        //     new Error(
        //       `Unsubscribing failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?`
        //     )
        //   );
        //   return;
        // }

        const hash = "todo";
        console.log("H", hash);

        // if (activeReqs.get())

        dispatch({
          op,
          resolve,
          reject,
          abort,
        });
      },
      abort() {
        // TODO: Attempt to remove from batch if still in it

        abort.abort();
      },
    };
  };
}

function generateHeaders(
  opts: HttpLinkOpts,
  arg: { op: Operation } | { ops: Operation[] }
) {
  const defaultHeaders =
    typeof opts.headers === "function"
      ? //@ts-expect-error Typescript narrowing is too hard for this.
        opts.headers(arg)
      : opts.headers;

  let headers = new Headers();
  if (defaultHeaders) {
    for (const [key, value] of Object.entries(defaultHeaders)) {
      if (Array.isArray(value)) {
        for (const v of value) {
          headers.append(key, v);
        }
      } else {
        headers.set(key, value || "");
      }
    }
  }

  return headers;
}

async function doFetch<T>(
  fetchFn: typeof fetch,
  url: string,
  init: RequestInit,
  // Signal is not used for batches
  signal?: AbortSignal
): Promise<T | RSPCError | undefined> {
  const resp = await fetchFn(url, init);
  if (resp.status !== 200) {
    return new RSPCError(500, "server responded with non-200 status");
  }

  if (resp.headers.get("Content-Type") !== "application/json") {
    return new RSPCError(500, "server responded with non-json response");
  }

  let body: T;
  try {
    body = await resp.json();
  } catch (err) {
    return new RSPCError(500, "server responded with invalid-json response");
  }

  if (signal?.aborted) {
    return;
  }

  return body;
}

// Generate the params for a non-batch request
function requestParams(
  opts: HttpLinkOpts,
  { op, abort }: BatchedItem
): [string, RequestInit] {
  const headers = generateHeaders(opts, { op: op });

  let url = encodeURIComponent(op.path);
  let body = undefined;
  if (op.method === "query" && op.input !== undefined) {
    const params = new URLSearchParams({
      input: JSON.stringify(op.input),
    });
    url += "?" + params.toString();
  } else if (op.method === "mutation") {
    headers.set("Content-Type", "application/json");
    body = JSON.stringify(op.input || {});
  }

  return [
    url,
    {
      method: op.method === "mutation" ? "POST" : "GET",
      headers,
      body,
      signal: abort.signal,
    },
  ];
}
