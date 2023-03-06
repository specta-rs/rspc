import { assert, expect, test, vi } from "vitest";
import { createClient, FetchTransport, WebsocketTransport } from "./index";

// Export from Rust. Run `cargo run -p example-axum` while running these tests for the server!
import type { Procedures } from "../../../examples/bindings";

test("Fetch Client", async () => {
  const client = createClient<Procedures>({
    transport: new FetchTransport("http://localhost:4000/rspc"),
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
  //   expect(async () => await client.query(["error"])).toThrowError("helloWorld");
  //   expect(async () => await client.mutation(["error"])).toThrowError(
  //     "helloWorld"
  //   );

  // TODO: Make this test work
  //   expect(async () => {
  //     await client.addSubscription(["pings"], {
  //       onData: (data) => {},
  //     });
  //     await new Promise((resolve) => setTimeout(resolve, 700)); // TODO: This is bad!
  //   }).toThrowError(
  //     "Error: Subscribing to 'pings' failed as the HTTP transport does not support subscriptions! Maybe try using the websocket transport?"
  //   );
});

test("Fetch Client (custom fetch function)", async () => {
  const client = createClient<Procedures>({
    transport: new FetchTransport("http://localhost:4000/rspc", (input, init) =>
      fetch(input, { ...init, headers: { "X-Demo-Header": "myCustomHeader" } })
    ),
  });

  expect(await client.query(["X-Demo-Header"])).toBe("myCustomHeader");
});

Object.assign(global, { WebSocket: require("ws") });

const timeout = (prom: Promise<any>, time: number) =>
  Promise.race([prom, new Promise((_r, rej) => setTimeout(rej, time))]);

test("Websocket Client", async () => {
  const client = createClient<Procedures>({
    transport: new WebsocketTransport("ws://localhost:4000/rspc/ws"),
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
  //   expect(async () => await client.query(["error"])).toThrowError("helloWorld");
  //   expect(async () => await client.mutation(["error"])).toThrowError(
  //     "helloWorld"
  //   );

  // TODO: Properly test websockets
  //   const onData = vi.fn();
  //   const y = timeout(async () => {
  //     await client.addSubscription(["pings"], {
  //       onData,
  //     });
  //   }, 5000);
  //   expect(onData).toHaveBeenCalledTimes(2);
});
