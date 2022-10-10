import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import solidJs from "@astrojs/solid-js";
import sitemap from "@astrojs/sitemap";
import compress from "astro-compress";
import rehypeExternalLinks from "rehype-external-links";

// https://astro.build/config
export default defineConfig({
  site: `https://rspc.dev`,
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
  integrations: [
    tailwind(),
    solidJs(),
    sitemap(),
    compress({
      html: {
        collapseWhitespace: true,
      },
    }),
  ],
  vite: {
    ssr: {
      external: ["svgo"],
    },
  },
});
