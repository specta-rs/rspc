import {
  fakeObservable,
  Link,
  OperationType,
  ProceduresDef,
  PromiseAndCancel,
  RSPCError,
} from "..";

// TODO: This shouldn't be exported but it is for now
// https://github.com/trpc/trpc/pull/669
export function arrayToDict(array: unknown[]) {
  const dict: Record<number, unknown> = {};
  for (let index = 0; index < array.length; index++) {
    const element = array[index];
    dict[index] = element;
  }
  return dict;
}

// TODO: Shouldn't be exported
export function getInput(opts: { inputs: unknown[] } | { input: unknown }) {
  return "input" in opts
    ? opts.input // opts.runtime.transformer.serialize(opts.input)
    : arrayToDict(
        opts.inputs // .map((_input) => opts.runtime.transformer.serialize(_input))
      );
}

// TODO: Get these from the rspc internal bindings
type TRPCResponse = any; // TODO
export interface HTTPResult {
  json: TRPCResponse;
  meta: {
    response: Response;
  };
}

function getWindow() {
  if (typeof window !== "undefined") {
    return window;
  }
  return globalThis;
}

function getFetch(f?: typeof fetch): typeof fetch {
  if (f) {
    return f;
  }

  const win = getWindow();
  const globalFetch = win.fetch;
  if (globalFetch) {
    return typeof globalFetch.bind === "function"
      ? globalFetch.bind(win)
      : globalFetch;
  }

  throw new Error("No fetch implementation found");
}

/**
 * @internal
 */
export type HTTPHeaders = Record<string, string | string[] | undefined>;

export interface FetchLinkOptions {
  url: string;
  /**
   * Add ponyfill for fetch
   */
  fetch?: typeof fetch;
  // TODO: Shouldn't this be ponyfilled at a higher level?
  // /**
  //  * Add ponyfill for AbortController
  //  */
  // AbortController?: typeof AbortController | null;
  /**
   * Headers to be set on outgoing requests or a callback that of said headers
   * @link http://trpc.io/docs/v10/header
   */
  headers?: HTTPHeaders | (() => HTTPHeaders | Promise<HTTPHeaders>);
  /**
   * Configure the credentials policy for the fetch call.
   * This is useful when you want to use cookies cross origin.
   */
  credentials?: RequestCredentials;
}

export function fetchLink<T extends ProceduresDef>(
  rawOpts: FetchLinkOptions
): Link<
  T,
  T,
  {
    terminatedLink: true;
    subscriptionsUnsupported: true;
  }
> {
  const opts: ResolvedFetchLinkOptions = {
    fetch: getFetch(rawOpts.fetch),
    // @ts-expect-error // TODO: Work out why it's complaining and fix it
    headers:
      typeof rawOpts.headers === "function"
        ? rawOpts.headers
        : rawOpts.headers || (() => ({})),
    ...rawOpts,
  };

  return (op) => {
    return fakeObservable(() => {
      const { path, input, type, context } = op.op;
      const { promise, cancel } = httpRequest({
        ...opts,
        fetch,
        type,
        path,
        input,
      });

      const p = promise.then((res) => {
        const transformed = transformResult(res.json);
        if (!transformed.ok) {
          const error = RSPCError.from(transformed.error, {
            meta: res.meta,
          });
          throw error;
        }

        return transformed;
      });

      return { promise: p, cancel };
    });
  };
}

type ResolvedFetchLinkOptions = {
  url: string;
  fetch: typeof fetch;
  headers: () => HTTPHeaders | Promise<HTTPHeaders>;
  credentials?: RequestCredentials;
};

// TODO: Name it
type TODO = {
  url: string;
  path: string;
  type: Omit<OperationType, "subscriptionStop">;
} & ({ inputs: unknown[] } | { input: unknown });

function getUrl(opts: TODO) {
  let url = opts.url + "/" + opts.path;
  const queryParts: string[] = [];
  if ("inputs" in opts) {
    queryParts.push("batch=1");
  }
  if (opts.type === "query") {
    const input = getInput(opts);
    if (input !== undefined) {
      queryParts.push(`input=${encodeURIComponent(JSON.stringify(input))}`);
    }
  }
  if (queryParts.length) {
    url += "?" + queryParts.join("&");
  }
  return url;
}

function getBody(
  opts: { type: Omit<OperationType, "subscriptionStop"> } & (
    | { inputs: unknown[] }
    | { input: unknown }
  )
) {
  if (opts.type === "query") {
    return undefined;
  }
  const input = getInput(opts);
  return input !== undefined ? JSON.stringify(input) : undefined;
}

const METHOD = {
  query: "GET",
  mutation: "POST",
} as const;

function httpRequest(
  opts: ResolvedFetchLinkOptions & TODO
): PromiseAndCancel<HTTPResult> {
  const { type } = opts;
  const ac = new AbortController(); // TODO: opts.AbortController ? new opts.AbortController() : null;

  const promise = new Promise<HTTPResult>((resolve, reject) => {
    const url = getUrl(opts);
    const body = getBody(opts);
    const meta = {} as HTTPResult["meta"];
    Promise.resolve(opts.headers())
      .then((headers) => {
        if (type === "subscription") {
          throw new Error("Subscriptions should use wsLink");
        }

        // TODO: Having to bind to `globalThis` shouldn't be required here because we do it earlier. Idk why this isn't working but it will work for now.
        return opts.fetch.bind(globalThis)(url, {
          // @ts-expect-error // TODO: Fix this
          method: METHOD[type],
          signal: ac?.signal,
          body: body,
          credentials: opts.credentials,
          headers: {
            "content-type": "application/json",
            ...headers,
          },
        });
      })
      .then((_res) => {
        meta.response = _res;
        return _res.json();
      })
      .then((json) => {
        resolve({
          json,
          meta,
        });
      })
      .catch(reject);
  });

  return {
    promise,
    cancel: () => {
      ac?.abort();
    },
  };
}

/**
 *
 * @internal
 */
// TODO: remove this from the public API or put into `@rspc/client/internal` export
export function transformResult(
  response: any // TODO: Type // TRPCResponseMessage<TOutput> | TRPCResponse<TOutput>
) {
  if (response.result.type === "error") {
    const error = response.result.data as any;
    return {
      ok: false,
      error: {
        ...response,
        error,
      },
    } as const;
  }

  const result = {
    ...response.result,
    ...((!response.result.type || response.result.type === "data") && {
      type: "data",
      data: response.result.data,
    }),
  } as any; // TODO: Types // TRPCResultMessage<TOutput>["result"];
  return { ok: true, result } as const;
}
