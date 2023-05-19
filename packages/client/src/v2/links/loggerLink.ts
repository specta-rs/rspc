import { Link } from "./link";

// TODO: Pretty log output like tRPC's logger link

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

    console.log("REQUEST", op, next);

    return {
      exec: (resolve, reject) => {
        result.exec(
          (data) => {
            console.log("RESPONSE", op, data);
            resolve(data);
          },
          (err) => {
            console.error("RESPONSE ERROR", op, err);
            reject(err);
          }
        );
      },
      abort: () => {
        console.log("ABORT OP", op);
        result.abort();
      },
    };
  };
}
