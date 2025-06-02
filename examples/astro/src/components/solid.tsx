/** @jsxImportSource solid-js */

import { createClient, fetchExecute, sseExecute, } from "@rspc/client/next";
import { createRSPCOptionsProxy } from "@rspc/solid-query";
import { QueryClient, QueryClientProvider, useQuery, useMutation } from "@tanstack/solid-query";

// Export from Rust. Run `cargo run -p example-axum` to start server and export it!
import { Procedures } from "../../../bindings";

const fetchQueryClient = new QueryClient();
// const url = "http://[::]:4000/rspc";
const url = "http://localhost:4000/rspc";
const client = createClient<Procedures>((args) => {
  if (args.type === "subscription") return sseExecute({ url }, args);
  else return fetchExecute({ url, batch: true, stream: true }, args);
})

export const rspc = createRSPCOptionsProxy<Procedures>(client);

function Example() {
  const version = useQuery(rspc.version.queryOptions(null))
  const validate = useQuery(rspc.validator.queryOptions({ mail: "test" }, { retry: false }))

  const mutation = useMutation(rspc.sendMsg.mutationOptions())

  return (
    <div>
      <h1>SolidJS</h1>
      <span>{version.data}</span>
      <button onClick={() => mutation.mutate("Message")}>Invalidate</button>
    </div>
  );
}

function App() {
  return (
    <QueryClientProvider client={fetchQueryClient}>
      <Example />
    </QueryClientProvider>
  );
}

export default App;
