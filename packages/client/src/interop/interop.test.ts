import { describe, it } from "vitest";
import { createClient, FetchTransport } from "../";
import { assertTy } from "../utils.test";

type Procedures = {
  queries:
    | { key: "a"; input: string; result: number }
    | { key: "b"; input: boolean; result: string }
    | { key: "c"; input: never; result: boolean };
  mutations:
    | { key: "d"; input: number; result: string }
    | { key: "e"; input: string; result: boolean }
    | { key: "f"; input: never; result: number };
  subscriptions:
    | { key: "g"; input: boolean; result: number }
    | { key: "h"; input: number; result: string }
    | { key: "i"; input: never; result: boolean };
};

const fetchClient = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

describe("interop API", () => {
  it("client queries", async () => {
    assertTy<number>(await fetchClient.query(["a", "hello"]));
    assertTy<string>(await fetchClient.query(["b", true]));
    assertTy<boolean>(await fetchClient.query(["c"]));
    // @ts-expect-error
    await fetchClient.query(["not a key"]);
  });

  it("client mutations", async () => {
    assertTy<string>(await fetchClient.mutation(["d", 42]));
    assertTy<boolean>(await fetchClient.mutation(["e", "hello"]));
    assertTy<number>(await fetchClient.mutation(["f"]));
    // @ts-expect-error
    await fetchClient.mutation(["not a key"]);
  });

  it("client subscriptions", async () => {
    fetchClient.addSubscription(["g", true], {
      onData: (data: number) => {},
    });
    fetchClient.addSubscription(["h", 42], {
      onData: (data: string) => {},
    });
    fetchClient.addSubscription(["i"], {
      onData: (data: boolean) => {},
    });
    // @ts-expect-error
    fetchClient.addSubscription(["not a key"], {
      onData: (data: any) => {},
    });
  });

  // TODO: NoOpTransport and WebsocketTransport both working
});
