/* @refresh reload */
import { render } from "solid-js/web";
import rspc, { client, queryClient } from "./query.tauri";
import App from "./App";

render(
  () => (
    <rspc.Provider client={client} queryClient={queryClient}>
      <App />
    </rspc.Provider>
  ),
  document.getElementById("root") as HTMLElement
);
