import { describe, it } from "vitest";
import { ProceduresDef, MiddlewareOperation, MiddlewareResp } from ".";

/**
 * TODO
 */
export function customLink() {
  return <T extends ProceduresDef>(
    op: MiddlewareOperation<T>
  ): MiddlewareResp<T, "subscriptionsUnsupported" | "terminatedLink"> => {
    const { path, input, type, context } = op; // TODO: Handle if a `batch` link was first cause there might be multiple.
    if (type === "subscription") {
      // We attempt to detect and prevent this error in the type system but it's not always possible to detect it.
      throw new Error("Subscriptions should use wsLink");
    }

    return (next) => {
      const x = new Promise((resolve, reject) => {
        setTimeout(() => {
          resolve("hello");
        }, 1000);
      });

      return next(x);
    };

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

describe("custom links", () => {
  it("todo", async () => {
    // TODO: Test crating custom links
  });
});
