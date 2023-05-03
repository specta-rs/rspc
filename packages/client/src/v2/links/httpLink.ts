import { AlphaRSPCError } from "../error";
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
  /**
   * Headers to be set on outgoing requests or a callback that of said headers
   */
  headers?:
    | HTTPHeaders
    | ((opts: { op: Operation }) => HTTPHeaders | Promise<HTTPHeaders>);
};

/**
 * HTTP Fetch link for rspc
 */
export function httpLink(opts: HttpLinkOpts): Link {
  const fetchFn = opts.fetch || globalThis.fetch.bind(globalThis);
  const abortController =
    opts.AbortController || globalThis.AbortController.bind(globalThis);

  return ({ op }) => {
    const abort = new abortController();
    return {
      exec: async (resolve, reject) => {
        if (op.type === "subscription" || op.type === "subscriptionStop") {
          reject(
            // TODO: Move to `AlphaRSPCError` type??
            new Error(
              `Subscribing to '${op.path}' failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?`
            )
          );
          return;
        }

        let method = "GET";
        let body = undefined as any;
        let headers = new Headers();

        const defaultHeaders =
          typeof opts.headers === "function"
            ? await opts.headers({ op })
            : opts.headers;
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

        const params = new URLSearchParams();
        if (op.type === "query") {
          if (op.input !== undefined) {
            params.append("input", JSON.stringify(op.input));
          }
        } else if (op.type === "mutation") {
          method = "POST";
          body = JSON.stringify(op.input || {});
          headers.set("Content-Type", "application/json");
        }

        const paramsStr = params.toString();
        const resp = await fetchFn(
          `${opts.url}/${op.path}${
            paramsStr.length > 0 ? `?${paramsStr}` : ""
          }`,
          {
            method,
            body,
            headers,
            signal: abort.signal,
          }
        );
        const respBody = await resp.json();
        const { type, data } = respBody.result;
        if (type === "error") {
          const { code, message } = data;
          reject(new AlphaRSPCError(code, message)); // TODO
          return;
        }

        resolve(data);
      },
      abort() {
        abort.abort();
      },
    };
  };
}
