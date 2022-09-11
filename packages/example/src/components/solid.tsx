/** @jsxImportSource solid-js */

import { createClient, FetchTransport } from "@rspc/client";
import { createSolidQueryHooks, QueryClient } from "@rspc/solid";
import { Operations } from "../../bindings";

const fetchQueryClient = new QueryClient();
const fetchClient = createClient<Operations>({
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const rspc = createSolidQueryHooks<Operations>();

function Example() {
  const echo = rspc.createQuery(["echo", "somevalue"]);
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
