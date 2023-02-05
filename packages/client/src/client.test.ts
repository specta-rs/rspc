import { describe, it } from "vitest";
import { T } from "vitest/dist/types-d97c72c7";
import { initRspc, noOpLink, ProceduresDef, LinkFlag, Link } from ".";
import { assertTy, HasProperty } from "./utils.test";

export function mapTypesLink<T extends ProceduresDef>(): Link<
  T,
  {
    queries: Omit<T["queries"], "a">;
    mutations: Omit<T["mutations"], "c">;
    subscriptions: Omit<T["subscriptions"], "e">;
  }
> {
  return undefined as any; // We only care about the types for this
}

export type Procedures = {
  queries:
    | { key: "a"; input: never; result: string }
    | { key: "b"; input: number; result: number };
  mutations:
    | { key: "c"; input: never; result: string }
    | { key: "d"; input: number; result: number };
  subscriptions:
    | { key: "e"; input: never; result: string }
    | { key: "f"; input: boolean; result: boolean };
};

describe("Client", () => {
  it("Typescript", async () => {
    // Without a terminating link you can't do operation and can add more links
    const c = initRspc<Procedures>();
    assertTy<HasProperty<typeof c, "use">, true>();
    assertTy<HasProperty<typeof c, "build">, true>();
    assertTy<HasProperty<typeof c, "query">, false>();
    assertTy<HasProperty<typeof c, "mutate">, false>();
    assertTy<HasProperty<typeof c, "subscribe">, false>();

    // With a Fetch terminating link you can do operations (other than subscriptions) but can't add more links
    const c2 = initRspc<Procedures>().use(
      noOpLink({ supportsSubscriptions: false })
    );
    assertTy<HasProperty<typeof c2, "use">, false>();
    assertTy<HasProperty<typeof c2, "build">, false>();
    assertTy<HasProperty<typeof c2, "query">, true>();
    assertTy<HasProperty<typeof c2, "mutate">, true>();
    assertTy<HasProperty<typeof c2, "subscribe">, false>();

    // With a Websocket terminating link you can do all operations but can't add more links
    const c3 = initRspc<Procedures>().use(
      noOpLink({ supportsSubscriptions: true })
    );
    assertTy<HasProperty<typeof c3, "use">, false>();
    assertTy<HasProperty<typeof c3, "build">, false>();
    assertTy<HasProperty<typeof c3, "query">, true>();
    assertTy<HasProperty<typeof c3, "mutate">, true>();
    assertTy<HasProperty<typeof c3, "subscribe">, true>();

    // Using build method enabling subscriptions
    const c4 = initRspc<Procedures>().unstable_build({
      supportsSubscriptions: true,
    });
    assertTy<HasProperty<typeof c4, "use">, true>();
    assertTy<HasProperty<typeof c4, "build">, false>();
    assertTy<HasProperty<typeof c4, "query">, true>();
    assertTy<HasProperty<typeof c4, "mutate">, true>();
    assertTy<HasProperty<typeof c4, "subscribe">, true>();

    // @ts-expect-error: We can't use a link which doesn't support subscriptions because we enabled them it in `.build()` call
    c4.use(noOpLink({ supportsSubscriptions: false })); // TODO: Assert error on this so we can be sure it's not something else
    c4.use(noOpLink({ supportsSubscriptions: true }));

    // // @ts-expect-error: Once built you can't apply a link which modifies the types
    // const y = c4.use(mapTypesLink()); // TODO: Type error because it changes the types once "built" flag exists
    // y.todo; // TODO

    // Using build method without enabling subscriptions
    const c5 = initRspc<Procedures>().unstable_build({
      supportsSubscriptions: false,
    });
    assertTy<HasProperty<typeof c5, "use">, true>();
    assertTy<HasProperty<typeof c5, "build">, false>();
    assertTy<HasProperty<typeof c5, "query">, true>();
    assertTy<HasProperty<typeof c5, "mutate">, true>();
    assertTy<HasProperty<typeof c5, "subscribe">, false>();

    c5.use(noOpLink({ supportsSubscriptions: true }));
    c5.use(noOpLink({ supportsSubscriptions: false }));

    // // // @ts-expect-error: Once built you can't apply a link which modifies the types
    // c4.use(mapTypesLink()); // TODO: Type error because it changes the types once "built" flag exists

    // TODO: Test `mapTypesLink` works on all the clients above
  });

  // TODO: Trying to set multiple terminating links will throw an error when using `.build()` syntax

  // TODO: Test a link which removes procedures, modify input/result types, etc
  // TODO: Global `onError` handler
  // TODO: Runtime pluggable transport
  // TODO: Test operation context works and order of execution on the links

  // TODO: Test stacking of flags. A middleware can ONLY add flags and not remove existing ones!
});
