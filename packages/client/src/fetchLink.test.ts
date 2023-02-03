import { describe, it, vi } from "vitest";
import { MiddlewareOperation, fetchLink, observableToPromise } from ".";

// TODO: Share between test files
// TODO: Renaming `key` to `path` in bindings
type BindingsOne = {
  queries:
    | { key: "a"; input: string; result: number }
    | { key: "b"; input: boolean; result: string };
  mutations:
    | { key: "c"; input: number; result: string }
    | { key: "d"; input: string; result: boolean };
  subscriptions:
    | { key: "e"; input: boolean; result: number }
    | { key: "f"; input: number; result: string };
};
// TODO: Bindings with all `never`, a single enum item in each and combination of types like above.

// TODO: Assert `Bindings` extends `ProceduresDef`

global.fetch = vi.fn();

describe("fetch link", () => {
  it("basic query", async () => {
    // @ts-ignore: TODO: Fix this
    fetch.mockResolvedValue({
      json: () => new Promise((resolve) => resolve("Hello World")),
    });

    const link = fetchLink({
      url: "/rspc",
    });

    const op: MiddlewareOperation<BindingsOne> = {
      type: "query",
      input: null, // TODO: null or undefined???
      path: "operationA",
      context: {},
    };

    const observable = link<BindingsOne>(op);

    const { promise, abort } = await observableToPromise(observable);

    abort();
    // console.log("RESULT", await promise);

    // TODO: Test `result.abort` works and calls AbortController

    // observable.subscribe();

    // const result = "123";
    // expect(result).toMatch("123");
  });

  // it("basic mutation", async () => {
  //   // TODO
  // });

  // TODO: Input or missing input for both mutation and query
  // TODO: Assert the next middleware is not being called -> Theorically impossible at type level
  // TODO: Assert resulting flags
  // TODO: Assert types are a link
  // TODO: Large query should be sent through request body
  // TODO: Allow setting custom HTTP headers and CORS cookies -> Overriding fetch
  // TODO: Subscription runtime error
  // TODO: Test abort controller is working
});
