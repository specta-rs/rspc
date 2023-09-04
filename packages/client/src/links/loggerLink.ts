import { ProceduresDef } from "../bindings";
import { Link } from "./link";

// TODO: Pretty log output like tRPC's logger link

export type LoggerLinkOpts = {
  enabled: boolean | (() => boolean);
};

/**
 * Link for logging operations.
 *
 * This must go before the terminating link for it to work!
 *
 */
export function loggerLink<P extends ProceduresDef>(
  opts?: LoggerLinkOpts
): Link<P> {
  const { enabled = true } = opts ?? {};
  const isEnabled = () => (typeof enabled === "function" ? enabled() : enabled);

  return ({ op, next }) => {
    const result = next({
      op,
    });

    if (isEnabled()) console.log("REQUEST", op, next);

    return {
      exec: (resolve, reject) => {
        result.exec(
          (data) => {
            if (isEnabled()) console.log("RESPONSE", op, data);
            resolve(data);
          },
          (err) => {
            if (isEnabled()) console.error("RESPONSE ERROR", op, err);
            reject(err);
          }
        );
      },
      abort: () => {
        if (isEnabled()) console.log("ABORT OP", op);
        result.abort();
      },
    };
  };
}
