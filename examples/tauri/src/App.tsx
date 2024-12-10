import { Channel } from "@tauri-apps/api/core";
import "./App.css";

import { handleRpc } from "@rspc/tauri";

function App() {
  const channel = new Channel();
  handleRpc(
    { method: "request", params: { path: "query", input: null } },
    channel,
  );
  channel.onmessage = console.log;

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid</h1>
    </main>
  );
}

export default App;
