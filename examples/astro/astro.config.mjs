import { defineConfig } from "astro/config";

import react from "@astrojs/react";
import solid from "@astrojs/solid-js";
import svelte from "@astrojs/svelte";

export default defineConfig({
  integrations: [
    react({
      include: ["**/react*"],
    }),
    solid({
      include: ["**/solid*"],
    }),
    svelte({
      include: ["**/svelte/*"],
    }),
  ],
  server: {
    port: 3000,
  },
});
