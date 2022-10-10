import { observable, ProcedureDef } from "..";
// import { TRPCClientError } from "../TRPCClientError";
import {
  HTTPLinkOptions,
  httpRequest,
  resolveHTTPLinkOptions,
} from "./internals/httpUtils";
import { transformResult } from "./internals/transformResult";
import { TRPCLink } from "./types";

export function httpLink<TProcedures extends ProcedureDef>(
  opts: HTTPLinkOptions
): TRPCLink<TProcedures> {
  const resolvedOpts = resolveHTTPLinkOptions(opts);
  return (runtime) =>
    ({ op }) =>
      observable((observer) => {
        const { path, input, type } = op;
        const { promise, cancel } = httpRequest({
          ...resolvedOpts,
          runtime,
          type,
          path,
          input,
        });
        promise
          .then((res) => {
            const transformed = transformResult(res.json, runtime);

            if (!transformed.ok) {
              // observer.error(
              //   TRPCClientError.from(transformed.error, {
              //     meta: res.meta,
              //   })
              // );
              throw new Error("BRUH1");
              return;
            }
            observer.next({
              context: res.meta,
              result: transformed.result,
            });
            observer.complete();
          })
          .catch((cause) => {
            console.error(cause);
            throw new Error("BRUH2");
            // observer.error(TRPCClientError.from(cause));
          });

        return () => {
          cancel();
        };
      });
}
