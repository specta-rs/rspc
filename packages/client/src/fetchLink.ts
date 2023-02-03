import { MiddlewareOperation, MiddlewareResp, ProceduresDef } from ".";

/**
 * TODO
 */
export interface FetchOpts {
  url: string;
  /**
   * Add ponyfill for fetch
   */
  fetch?: typeof globalThis.fetch;
  /**
   * Add ponyfill for AbortController
   */
  // AbortController?: typeof AbortController | null;
  // headers?: HTTPHeaders | (() => HTTPHeaders | Promise<HTTPHeaders>);
  // TODO: Shortcut for cross site cookies
}

/**
 * TODO
 */
export function fetchLink(opts: FetchOpts) {
  const globalFetch = opts.fetch || globalThis.fetch;
  // headers: typeof headers === "function" ? headers : () => headers, // TODO

  return <T extends ProceduresDef>(
    op: MiddlewareOperation<T>
  ): MiddlewareResp<T, "subscriptionsUnsupported" | "terminatedLink"> => {
    const { path, input, type, context } = op; // TODO: Handle if a `batch` link was first cause there might be multiple.
    if (type === "subscription") {
      // We attempt to detect and prevent this error in the type system but it's not always possible to detect it.
      throw new Error("Subscriptions should use wsLink");
    }

    return {
      subscribe(observer) {
        const url = internal_fetchLinkGetUrl(opts.url, op);
        const options = {}; // TODO
        const resp = globalFetch(url, options).then(async (resp) => {
          // TODO: Handle status code and errors

          try {
            const data = await resp.json();
            observer.next(data);
          } catch (err) {
            // observer.error(err); // TODO
            throw err; // TODO: Remove
          }
        });

        return () => {
          console.log("CLEANUP");
          // TODO: Abort controller
        };
      },
    };
  };
}

/**
 * TODO
 *
 * @internal
 */
export function internal_fetchLinkGetUrl(
  baseURL: string,
  opts: MiddlewareOperation<any>
) {
  let url = baseURL + "/" + opts.path;
  const queryParts: string[] = [];
  // TODO
  // if ("inputs" in opts) {
  //   queryParts.push("batch=1");
  // }
  // if (opts.type === "query") {
  //   const input = getInput(opts);
  //   if (input !== undefined) {
  //     queryParts.push(`input=${encodeURIComponent(JSON.stringify(input))}`);
  //   }
  // }
  if (queryParts.length) {
    url += "?" + queryParts.join("&");
  }
  return url;
}
