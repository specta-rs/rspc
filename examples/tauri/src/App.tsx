import { createClient as createLegacyClient } from "@rspc/client";
import { TauriTransport } from "@rspc/tauri";

import { createClient } from "@rspc/client/next";
import { tauriExecute } from "@rspc/tauri/next";

import { Procedures, ProceduresLegacy } from "../../bindings";

import "./App.css";

const legacyClient = createLegacyClient<ProceduresLegacy>({
	transport: new TauriTransport(),
});
const client = createClient<Procedures>(tauriExecute);

function App() {
	client.sendMsg.mutate("bruh").then(console.log);

	legacyClient.mutation(["sendMsg", "bruh2"]).then(console.log);
	legacyClient.addSubscription(["basicSubscription", null], {
		onData: (d) => {
			console.log("subscription", d);
		},
	});

	return (
		<main class="container">
			<h1>Welcome to Tauri + Solid</h1>
		</main>
	);
}

export default App;
