import { Link } from "./link";

/**
 * Link for batching operations into a single request to the backend.
 *
 * This must go before the terminating link for it to work!
 *
 */
export function batchLink(): Link {
  return ({ op, next }) => {
    const result = next({
      op,
    });

    return {
      exec: (resolve, reject) => {
        result.exec((data) => {
          console.log("BATCH", data); // TODO
          resolve(data);
        }, reject);
      },
      abort: result.abort,
    };
  };
}
