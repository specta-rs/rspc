/** @jsxImportSource solid-js */

import { createClient, FetchTransport } from "@rspc/client";
import { createSolidQueryHooks } from "@rspc/solid";
import { QueryClient } from "@tanstack/solid-query";
import { Procedures } from "../../../bindings";

const fetchQueryClient = new QueryClient();
const fetchClient = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const rspc = createSolidQueryHooks<Procedures>();

function Example() {
  const echo = rspc.createQuery(() => ["echo", "somevalue"]);
  const sendMsg = rspc.createMutation("sendMsg");

  sendMsg.mutate("Sending");

  return (
    <div style="background-color: rgba(255, 105, 97, .5);">
      <h1>SolidJS</h1>
      <p>{echo.data}</p>
      {/* TODO: Finish SolidJS example */}
    </div>
  );
}

function App() {
  return (
    <rspc.Provider client={fetchClient} queryClient={fetchQueryClient}>
      <Example />
    </rspc.Provider>
  );
}

export default App;
