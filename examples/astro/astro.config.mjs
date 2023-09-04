import { defineConfig } from "astro/config";
import react from "@astrojs/react";
import solid from "@astrojs/solid-js";
import svelte from "@astrojs/svelte";
import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  integrations: [
    react({
      include: ["**/react/*"],
    }),
    solid({
      include: ["**/solid/*"],
    }),
    svelte(),
    tailwind(),
  ],
  server: {
    port: 3000,
  },
});
