export const SITE = {
  title: "rspc",
  description: "A blazing fast and easy to use TRPC-like server for Rust.",
  defaultLanguage: "en_US",
};

export const GITHUB_EDIT_URL = `https://github.com/oscartbeaumont/rspc/blob/main/docs/`;

export const SIDEBAR = [
  { text: "", header: true },
  { text: "rspc", header: true },
  { text: "Introduction", link: "" },
  { text: "Quickstart", link: "quickstart" },
  { text: "Roadmap", link: "roadmap" },

  { text: "Server", header: true },
  { text: "Define router", link: "server/define-routers" },
  { text: "Merging routers", link: "server/merge-routers" },
  { text: "Request context", link: "server/request-context" },
  { text: "Middleware", link: "server/middleware" },
  { text: "Route metadata", link: "server/router-metadata" },
  { text: "Selection", link: "server/selection" },
  { text: "Error handling", link: "server/error-handling" },

  { text: "Client", header: true },
  { text: "Create client", link: "client/create-client" },

  { text: "Integrations", header: true },
  { text: "Axum", link: "integrations/axum" },
  { text: "Tauri", link: "integrations/tauri" },
];
