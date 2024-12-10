import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import { handleRpc } from "@rspc/tauri";

function App() {
	handleRpc({ method: "request", params: { path: "query", input: null } });

	return (
		<main class="container">
			<h1>Welcome to Tauri + Solid</h1>
		</main>
	);
}

export default App;
