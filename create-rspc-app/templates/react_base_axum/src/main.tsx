import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { rspc, client, queryClient } from "./utils/rspc";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <rspc.Provider client={client} queryClient={queryClient}>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </rspc.Provider>
  </React.StrictMode>
);
