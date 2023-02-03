// TODO: Remove all this

import { describe, it } from "vitest";
import { initRspc, MiddlewareResp } from "../client2";

// function batch() {
//   return <T>(p: T): MiddlewareResp<T> => {
//     return {};
//   };
// }

// function demo() {
//   return <T>(p: T): MiddlewareResp<Exclude<T, { key: "a" }>> => {
//     return {};
//   };
// }

describe("incubator", () => {
  it("link-chains", async () => {
    type Bindings = {
      queries:
        | { key: "a"; input: string; result: number }
        | { key: "b"; input: boolean; result: string };
      mutations: never;
      subscriptions: never;
    };

    const a = initRspc<Bindings>();

    // a.query("a");
    // a.query("b");
    // a.subscribe();

    // const b = a.use(batch());

    // b.query("b");
    // b.query("b");
    // b.subscribe();

    // const c = b.use(fetch());

    // // @ts-expect-error because fetch transport removed the "a" key. This is just an example.
    // c.query("a");
    // c.query("b");
    // // @ts-expect-error because fetch transport doesn't support subscriptions.
    // c.subscribe();

    // // @ts-expect-error because fetch transport is a terminating link you can't attach any more.
    // const d = c.use(demo());
  });
});
