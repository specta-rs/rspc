import { Link } from "./link";

/**
 * Link for logging operations.
 *
 * This must go before the terminating link for it to work!
 *
 */
export function loggerLink(): Link {
  return ({ op, next }) => {
    const result = next({
      op,
    });

    return {
      exec: (resolve, reject) => {
        result.exec((data) => {
          console.log("LOGGER", data); // TODO
          resolve(data);
        }, reject);
      },
      abort: result.abort,
    };
  };
}
