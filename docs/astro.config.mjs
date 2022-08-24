import { defineConfig } from "astro/config";
import preact from "@astrojs/preact";
import rehypeExternalLinks from "rehype-external-links";
import astro from "astro-compress";

// https://astro.build/config
export default defineConfig({
  site: "https://rspc.otbeaumont.me",
  integrations: [
    // Enable Preact to support Preact JSX components.
    preact(),
    astro(),
  ],
  markdown: {
    rehypePlugins: [
      [
        rehypeExternalLinks,
        {
          target: "_blank",
          rel: ["nofollow"],
        },
      ],
    ],
  },
  vite: {
    ssr: {
      external: ["svgo"],
    },
  },
});
