import { observable, ProcedureDef, RSPCError } from "..";
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
        const { path, input, type, context } = op;
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
              const error = RSPCError.from(transformed.error, {
                meta: res.meta,
              });
              // TODO
              runtime.onError?.({
                error,
                path,
                input,
                ctx: context,
                type: type,
              });

              observer.error(error);
              return;
            }
            observer.next({
              context: res.meta,
              result: transformed.result,
            });
            observer.complete();
          })
          .catch((cause) => {
            const error = RSPCError.from(cause);
            // TODO
            runtime.onError?.({
              error,
              path,
              input,
              ctx: context,
              type: type,
            });

            observer.error(error);
          });

        return () => {
          cancel();
        };
      });
}
