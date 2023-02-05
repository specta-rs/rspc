// import { createRspcRoot, createWSClient, httpLink, wsLink } from "@rspc/client";
// import { createRspcReact } from "@rspc/react";
// import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
// import React, { useState } from "react";

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

// export const rspcReact = createRspcReact<typeof fetchClient>();
// const rspc = rspcReact.createHooks();

// function Example({ name }: { name: string }) {
//   const [rerenderProp, setRendererProp] = useState(Date.now().toString());
//   const { data: version } = rspc.useQuery(["version"]);
//   const { data: transformMe } = rspc.useQuery(["basic.transformMe"]);
//   const { data: echo } = rspc.useQuery(["basic.echo", "Hello From Frontend!"]);
//   const { mutate, isLoading } = rspc.useMutation("basic.sendMsg");
//   const { error } = rspc.useQuery(["basic.error"], {
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
//       <h1>{name}</h1>
//       <p>Using rspc version: {version}</p>
//       <p>Echo response: {echo}</p>
//       <p>
//         Error returned: {error?.code} {error?.message}
//       </p>
//       <p>Transformed Query: {transformMe}</p>
//       <ExampleSubscription rerenderProp={rerenderProp} />
//       <button onClick={() => setRendererProp(Date.now().toString())}>
//         Rerender subscription
//       </button>
//       <button onClick={() => mutate("Hello!")} disabled={isLoading}>
//         Send Msg!
//       </button>
//     </div>
//   );
// }

// function ExampleSubscription({ rerenderProp }: { rerenderProp: string }) {
//   const [i, setI] = useState(0);
//   rspc.useSubscription(["subscriptions.pings"], {
//     onData(msg) {
//       setI((i) => i + 1);
//     },
//   });

//   return (
//     <p>
//       Pings received: {i} {rerenderProp}
//     </p>
//   );
// }

export default function App() {
  //   return (
  //     <React.StrictMode>
  //       <div
  //         style={{
  //           backgroundColor: "rgba(50, 205, 50, .5)",
  //         }}
  //       >
  //         <h1>React</h1>
  //         <rspcReact.Provider client={fetchClient} queryClient={fetchQueryClient}>
  //           <Example name="Fetch Transport" />
  //         </rspcReact.Provider>
  //         <rspcReact.Provider client={wsClient} queryClient={wsQueryClient}>
  //           <Example name="Websocket Transport" />
  //         </rspcReact.Provider>
  //       </div>
  //     </React.StrictMode>
  //   );

  return null; // TODO
}
