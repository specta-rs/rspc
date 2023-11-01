import { expect, test } from "vitest";
import { createRSPCClient, httpLink, wsLink } from "./index";
import { makeDoesntSupportSubscriptionsError } from "./links/httpLink";

// Export from Rust. Run `cargo run -p example-axum` while running these tests for the server!
import type { Procedures } from "../../../examples/bindings";

test("Fetch Client", async () => {
  const client = createRSPCClient({
    links: [httpLink({ url: "http://localhost:4000/rspc" })],
  });

  function dontRun() {
    // @ts-expect-error
    client.query([]);
    // @ts-expect-error
    client.query(["invalidKey"]);
    // @ts-expect-error
    client.query(["echo", 42]);

    // @ts-expect-error
    client.mutation([]);
    // @ts-expect-error
    client.mutation(["invalidKey"]);
    // @ts-expect-error
    client.mutation(["sendMsg", 42]);
  }

  expect(await client.query(["version"])).toBe("0.0.0");
  expect(await client.query(["echo", "Demo"])).toBe("Demo");

  expect(await client.mutation(["sendMsg", "helloWorld"])).toBe("helloWorld");

  // TODO
  expect(() => client.query(["error"])).toThrowError("helloWorld");
  expect(() => client.mutation(["error"])).toThrowError("helloWorld");

  // TODO: Make this test work
  expect(async () => {
    client.addSubscription(["pings"], {
      onData: (data) => {},
    });
    await new Promise((resolve) => setTimeout(resolve, 700)); // TODO: This is bad!
  }).toThrowError(makeDoesntSupportSubscriptionsError("pings"));
});

test("Fetch Client (custom fetch function)", async () => {
  const client = new Client<Procedures>({
    links: [
      httpLink({
        url: "http://localhost:4000/rspc",
        fetch: (input, init) =>
          fetch(input, {
            ...init,
            headers: { "X-Demo-Header": "myCustomHeader" },
          }),
      }),
    ],
  });

  expect(await client.query(["X-Demo-Header"])).toBe("myCustomHeader");
});

Object.assign(global, { WebSocket: require("ws") });

const timeout = (prom: Promise<any>, time: number) =>
  Promise.race([prom, new Promise((_r, rej) => setTimeout(rej, time))]);

test("Websocket Client", async () => {
  const client = new Client<Procedures>({
    links: [wsLink({ url: "ws://localhost:4000/rspc/ws" })],
  });

  function dontRun() {
    const opts = {
      onData: (data: any) => {},
    };
    client.addSubscription(["pings"], opts);

    // @ts-expect-error
    client.addSubscription([], opts);
    // @ts-expect-error
    client.addSubscription(["invalidKey"], opts);
  }

  expect(await client.query(["version"])).toBe("0.0.0");
  expect(await client.query(["echo", "Demo"])).toBe("Demo");

  expect(await client.mutation(["sendMsg", "helloWorld"])).toBe("helloWorld");

  // TODO: Unit test errors
  expect(() => client.query(["error"])).toThrowError("helloWorld");
  expect(() => client.mutation(["error"])).toThrowError("helloWorld");

  // TODO: Properly test websockets
  // const onData = vi.fn();
  // const y = timeout(() => {
  //   client.addSubscription(["pings"], {
  //     onData,
  //   });
  // }, 5000);
  // expect(onData).toHaveBeenCalledTimes(2);
});

// TODO: Test with and without batching
