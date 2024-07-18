import { defineConfig } from 'astro/config';
import solidJs from "@astrojs/solid-js";
import react from "@astrojs/react";
import svelte from "@astrojs/svelte";

import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  integrations: [solidJs(), react(), svelte(), tailwind()]
});