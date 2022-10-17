import {
  createWSClient,
  HookCreateFunction,
  HookOptions,
  ProcedureDef,
  ProceduresDef,
  wsLink,
} from "@rspc/client";
// import { createReactHooks } from "@rspc/react";
// import { createNormiHooks } from "@rspc/normi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";

import type { Procedures } from "../../../bindings";

// const libraryHooks: HookCreateFunction = <
//   TBaseProcedures extends ProceduresDef = never,
//   TQueries extends ProcedureDef = TBaseProcedures["queries"],
//   TMutations extends ProcedureDef = TBaseProcedures["mutations"],
//   TSubscriptions extends ProcedureDef = TBaseProcedures["subscriptions"]
// >(
//   opts: HookOptions = {}
// ) =>
//   createReactHooks<TBaseProcedures, TQueries, TMutations, TSubscriptions>({
//     internal: {
//       customHooks() {
//         const nextHooks = opts.internal?.customHooks?.();

//         return {
//           useQuery: (keyAndInput, next) => {
//             // console.log("SD HOOKS: ", keyAndInput);
//             return (
//               nextHooks?.useQuery?.(keyAndInput, next) || next(keyAndInput)
//             );
//           },
//         };
//       },
//     },
//   });

// // TODO: Stack normi custom hooks and Spacedrive custom hooks
// const rspc = createNormiHooks<Procedures>(createReactHooks, {
//   contextSharing: true,
// });

// export const rspc = createReactHooks<
//   Procedures,
//   Exclude<Normalized<Procedures["queries"]>, { key: "version" }>,
//   Normalized<Procedures["mutations"]>,
//   Normalized<Procedures["subscriptions"]>
// >({
//   internal: {
//     customHooks: () => {
//       // TODO: Access to query client here.
//       // TODO: Subscribing to backend alerts for changes
//       // TODO: Disable staleness in React Query

//       return {
//         async useQuery(keyAndInput, next) {
//           console.log("Hello World");
//           const data = await next(keyAndInput);
//           console.log("AA", keyAndInput, data);
//           return data;
//         },
//         // TODO: useMutation
//         // TODO: useSubscription
//       };
//     },
//   },
// });

// function Demo() {
//   const { data: version } = rspc.useQuery(["version"]);
//   const { data: user } = rspc.useQuery(["user"]);

//   return (
//     <>
//       <h1>Hello World</h1>
//       <p>Version: {version}</p>
//       <p>User: {JSON.stringify(user)}</p>
//     </>
//   );
// }

// const queryClient = new QueryClient();
// const wsClient = createWSClient({
//   url: "ws://localhost:4000/rspc/ws",
// });

// const client = rspc.createClient({
//   links: [
//     wsLink({
//       client: wsClient,
//     }),
//   ],
// });

export default function App() {
  return (
    <React.StrictMode>
      <div
        style={{
          backgroundColor: "rgba(50, 205, 50, .5)",
        }}
      >
        <h1>React</h1>
        {/* <rspc.Provider client={client} queryClient={queryClient}>
          <Demo />
        </rspc.Provider> */}
      </div>
    </React.StrictMode>
  );
}
