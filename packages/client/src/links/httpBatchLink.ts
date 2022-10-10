import { observable } from "../observable";
import { dataLoader } from "../internals/dataLoader";
import {
  HTTPLinkOptions,
  HTTPResult,
  getUrl,
  httpRequest,
  resolveHTTPLinkOptions,
} from "./internals/httpUtils";
import { transformResult } from "./internals/transformResult";
import { TRPCLink } from "./types";
import { ProceduresDef } from "..";

type ProcedureType = any; // TODO

export interface HttpBatchLinkOptions extends HTTPLinkOptions {
  maxURLLength?: number;
}

export function httpBatchLink<TProcedures extends ProceduresDef>(
  opts: HttpBatchLinkOptions
): TRPCLink<TProcedures> {
  const resolvedOpts = resolveHTTPLinkOptions(opts);
  // initialized config
  return (runtime) => {
    type BatchOperation = { id: number; path: string; input: unknown };

    const maxURLLength = opts.maxURLLength || Infinity;

    const batchLoader = (type: ProcedureType) => {
      const validate = (batchOps: BatchOperation[]) => {
        if (maxURLLength === Infinity) {
          // escape hatch for quick calcs
          return true;
        }
        const path = batchOps.map((op) => op.path).join(",");
        const inputs = batchOps.map((op) => op.input);

        const url = getUrl({
          ...resolvedOpts,
          runtime,
          type,
          path,
          inputs,
        });
        return url.length <= maxURLLength;
      };

      const fetch = (batchOps: BatchOperation[]) => {
        const path = batchOps.map((op) => op.path).join(",");
        const inputs = batchOps.map((op) => op.input);

        const { promise, cancel } = httpRequest({
          ...resolvedOpts,
          runtime,
          type,
          path,
          inputs,
        });

        return {
          promise: promise.then((res) => {
            const resJSON = Array.isArray(res.json)
              ? res.json
              : batchOps.map(() => res.json);

            const result = resJSON.map((item) => ({
              meta: res.meta,
              json: item,
            }));

            return result;
          }),
          cancel,
        };
      };

      return { validate, fetch };
    };

    const query = dataLoader<BatchOperation, HTTPResult>(batchLoader("query"));
    const mutation = dataLoader<BatchOperation, HTTPResult>(
      batchLoader("mutation")
    );
    const subscription = dataLoader<BatchOperation, HTTPResult>(
      batchLoader("subscription")
    );

    const loaders = { query, subscription, mutation };
    return ({ op }) => {
      return observable((observer) => {
        const loader = loaders[op.type];
        const { promise, cancel } = loader.load(op);

        promise
          .then((res) => {
            const transformed = transformResult(res.json, runtime);

            if (!transformed.ok) {
              // observer.error(
              //   TRPCClientError.from(transformed.error, {
              //     meta: res.meta,
              //   })
              // );
              throw new Error("BRUH5"); // TODO
              return;
            }
            observer.next({
              context: res.meta,
              result: transformed.result,
            });
            observer.complete();
          })
          .catch((err) => {
            // observer.error(TRPCClientError.from(err))
            throw new Error("BRUH6"); // TODO
          });

        return () => {
          cancel();
        };
      });
    };
  };
}
