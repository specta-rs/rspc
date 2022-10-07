import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import rspc, { client, queryClient } from "./query.tauri";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <rspc.Provider client={client} queryClient={queryClient}>
      <App />
    </rspc.Provider>
  </React.StrictMode>
);
