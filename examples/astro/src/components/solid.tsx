// /** @jsxImportSource solid-js */
// import { createRspcRoot, createWSClient, httpLink, wsLink } from "@rspc/client";
// import { createRspcSolid } from "@rspc/solid";
// import { QueryClient } from "@tanstack/solid-query";
// import { createSignal } from "solid-js";

// import type { Procedures } from "../../../bindings";

// const root = createRspcRoot<Procedures>();

// const fetchClient = root.createClient({
//   // onError(opts) {
//   //   console.error("A", opts);
//   // },
//   links: [
//     // loggerLink(),
//     httpLink({
//       url: "http://localhost:4000/rspc",
//     }),
//     // TODO: Support batching server-side
//     // httpBatchLink({
//     //   url: "http://localhost:4000/rspc",
//     // }),
//   ],
// });
// export const fetchQueryClient = new QueryClient();

// const wsClient = root.createClient({
//   // onError(opts) {
//   //   console.error("B", opts);
//   // },
//   links: [
//     // loggerLink(),
//     wsLink({
//       client: createWSClient({
//         url: "ws://localhost:4000/rspc/ws",
//       }),
//     }),
//   ],
// });
// export const wsQueryClient = new QueryClient();

// export const rspcSolid = createRspcSolid<typeof fetchClient>();
// const rspc = rspcSolid.createHooks();

// function Example(props: { name: string }) {
//   const [rerenderProp, setRendererProp] = createSignal(Date.now().toString());
//   const version = rspc.createQuery(() => ["version"]);
//   const transformMe = rspc.createQuery(() => ["basic.transformMe"]);
//   const echo = rspc.createQuery(() => ["basic.echo", "Hello From Frontend!"]);
//   const sendMsg = rspc.createMutation("basic.sendMsg");
//   const error = rspc.createQuery(() => ["basic.error"], {
//     retry: false,
//     onSuccess(v) {
//       console.log("WHY", v);
//     },
//     onError(err) {
//       console.error("A", err);
//     },
//   });

//   return (
//     <div
//       style={{
//         border: "black 1px solid",
//       }}
//     >
//       <h1>{props.name}</h1>
//       <p>Using rspc version: {version.data}</p>
//       <p>Echo response: {echo.data}</p>
//       <p>
//         Error returned: {error.error?.code} {error.error?.message}
//       </p>
//       <p>Transformed Query: {transformMe.data}</p>
//       <ExampleSubscription rerenderProp={rerenderProp()} />
//       <button onClick={() => setRendererProp(Date.now().toString())}>
//         Rerender subscription
//       </button>
//       <button
//         onClick={() => sendMsg.mutate("Hello!")}
//         disabled={sendMsg.isLoading}
//       >
//         Send Msg!
//       </button>
//     </div>
//   );
// }

// function ExampleSubscription(props: { rerenderProp: string }) {
//   const [i, setI] = createSignal(0);
//   rspc.createSubscription(() => ["subscriptions.pings"], {
//     onData(msg) {
//       console.log("SUBSCRIPTION: ", msg);
//       setI((i) => i + 1);
//     },
//   });

//   return (
//     <p>
//       Pings received: {i} {props.rerenderProp}
//     </p>
//   );
// }

export default function App() {
  //   return (
  //     <div style="background-color: rgba(255, 105, 97, .5);">
  //       <h1>Solid</h1>
  //       <rspcSolid.Provider client={fetchClient} queryClient={fetchQueryClient}>
  //         <Example name="Fetch Transport" />
  //       </rspcSolid.Provider>
  //       <rspcSolid.Provider client={wsClient} queryClient={wsQueryClient}>
  //         <Example name="Websocket Transport" />
  //       </rspcSolid.Provider>
  //     </div>
  //   );

  return null; // TODO
}
