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
  { text: "Breaking changes", link: "breaking-changes" },
  { text: "Related projects", link: "related" },

  { text: "Server", header: true },
  { text: "Router", link: "server/router" },
  { text: "Specta", link: "specta" },
  { text: "Request context", link: "server/request-context" },
  { text: "Middleware", link: "server/middleware" },
  { text: "Selection", link: "server/selection" },
  { text: "Error handling", link: "server/error-handling" },
  { text: "Convention", link: "server/convention" },
  { text: "Common errors", link: "server/common-errors" },
  { text: "Deployment", link: "server/deployment" },

  { text: "Client", header: true },
  { text: "Client", link: "client" },
  { text: "TanStack Query", link: "client/tanstack-query" },

  { text: "Integrations", header: true },
  { text: "Axum", link: "integrations/axum" },
  { text: "Tauri", link: "integrations/tauri" },
];
