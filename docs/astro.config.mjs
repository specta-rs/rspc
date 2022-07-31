import { defineConfig } from "astro/config";
import preact from "@astrojs/preact";
import rehypeExternalLinks from "rehype-external-links";

// https://astro.build/config
export default defineConfig({
  site: "https://rspc.otbeaumont.me",
  integrations: [
    // Enable Preact to support Preact JSX components.
    preact(),
  ],
  markdown: {
    rehypePlugins: [
      [rehypeExternalLinks, { target: "_blank", rel: ["nofollow"] }],
    ],
  },
  vite: {
    ssr: {
      external: ["svgo"],
    },
  },
});
