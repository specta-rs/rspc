/** @jsxImportSource solid-js */

import { createClient, fetchExecute, sseExecute, } from "@rspc/client/next";
import { createRSPCOptionsProxy, inferInput, inferOutput, useSubscription, } from "@rspc/solid-query";
import { QueryClient, QueryClientProvider, useQuery, useMutation, skipToken } from "@tanstack/solid-query";
import { Show } from "solid-js";

// Export from Rust. Run `cargo run -p example-axum` to start server and export it!
import { Procedures } from "../../../bindings";

const fetchQueryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false
    }
  }
});
const url = "http://localhost:4000/rspc";
const client = createClient<Procedures>((args) => {
  if (args.type === "subscription") return sseExecute({ url }, args);
  else return fetchExecute({ url, batch: true, stream: true }, args);
})

const rspc = createRSPCOptionsProxy<Procedures>(client);

function Example() {
  type Version = inferOutput<typeof rspc.version>
  const version = useQuery(rspc.version.queryOptions())
  const validate = useQuery(rspc.validator.queryOptions({ mail: "example@example.com" }))

  const mutation = useMutation(rspc.sendMsg.mutationOptions({
    onSettled() {
      fetchQueryClient.invalidateQueries({
        queryKey: rspc.version.queryKey()
      })
    },
  }))

  const subscription = useSubscription(rspc.basicSubscription.subscriptionOptions(null, {
    enabled: true,
    onData(value) {
      console.log("Data received", value)
    },
    onError(err) {
      console.error(err.type, err.error)
    },
  }))

  return (
    <div>
      <h1>SolidJS</h1>
      <Show when={!version.isLoading} fallback={<p>Loading</p>}>
        <p>{version.data}</p>
      </Show>
      <p>subscription {JSON.stringify(subscription.data)}</p>
      <Show when={subscription.error}>
        {error =>
          <p>Error {error().type}</p>
        }
      </Show>
      <p>status {subscription.status}</p>
      <button onClick={() => mutation.mutate("Message")}>Trigger mutation</button>
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
