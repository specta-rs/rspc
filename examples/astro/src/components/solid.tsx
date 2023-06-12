/** @jsxImportSource solid-js */

// import { createClient, FetchTransport } from "@rspc/client";
// import { createSolidQueryHooks } from "@rspc/solid";
import {
  QueryClient,
  QueryClientProvider,
  createQuery,
} from "@tanstack/solid-query";

// Export from Rust. Run `cargo run -p example-axum` to start server and export it!
// import { Procedures } from "../../../bindings";

const fetchQueryClient = new QueryClient();
// const fetchClient = createClient<Procedures>({
//   transport: new FetchTransport("http://localhost:4000/rspc"),
// });

// const rspc = createSolidQueryHooks<Procedures>();

// function Example() {
//   const echo = rspc.createQuery(() => ["echo", "somevalue"]);
//   const sendMsg = rspc.createMutation("sendMsg");

//   sendMsg.mutate("Sending");

//   return (
//     <div style="background-color: rgba(255, 105, 97, .5);">
//       <h1>SolidJS</h1>
//       <p>{echo.data}</p>
//       {/* TODO: Finish SolidJS example */}
//     </div>
//   );
// }

// function App() {
//   return null;
//   // {/* <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}> */}
//   // {/*   <Example /> */}
//   // {/* </rspc.Provider> */}
// }

// export default App;

export default () => (
  <QueryClientProvider client={fetchQueryClient}>
    <PLzWork />
  </QueryClientProvider>
);

function PLzWork() {
  console.log("SOLID INIT");
  const x = createQuery({
    queryKey: () => ["demo"],
    queryFn: () => {
      console.log("FIRE");
      return "plz work";
    },
  });

  return <h1>Hello Solid {x.data}</h1>;
}
