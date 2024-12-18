import { createClient } from "@rspc/client/next";
import { tauriExecute } from "@rspc/tauri/next";

import { Procedures } from "../../bindings";

import "./App.css";

const client = createClient<Procedures>(tauriExecute);

function App() {
	client.sendMsg.mutate("bruh").then(console.log);

	return (
		<main class="container">
			<h1>Welcome to Tauri + Solid</h1>
		</main>
	);
}

export default App;
