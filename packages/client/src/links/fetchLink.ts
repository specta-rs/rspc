// TODO: Fetch options

import { LinkFlag, Link, ProceduresDef, JoinLinkFlags } from "..";

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
  //   /**
  //    * Add ponyfill for AbortController
  //    */
  //   AbortController?: typeof AbortController | null;
  //   /**
  //    * Headers to be set on outgoing requests or a callback that of said headers
  //    * @link http://trpc.io/docs/v10/header
  //    */
  //   headers?: HTTPHeaders | (() => HTTPHeaders | Promise<HTTPHeaders>);
  //   credentials: "omit" | "same-origin" | "include";
}

export function fetchLink<T extends ProceduresDef>(
  opts: FetchLinkOptions
): Link<
  T,
  T,
  {
    terminatedLink: true;
    subscriptionsUnsupported: true;
  }
> {
  const fetch = opts.fetch || globalThis.fetch;

  return (op) => {
    // TODO
  };
}
