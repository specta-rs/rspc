import { defineConfig } from "astro/config";
import solid from "@astrojs/solid-js";
import rehypeExternalLinks from "rehype-external-links";
import astro from "astro-compress";
import sitemap from "@astrojs/sitemap";
import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  site: "https://rspc.otbeaumont.me",
  integrations: [
    astro(),
    solid(),
    sitemap(),
    tailwind({
      config: {
        applyBaseStyles: false,
      },
    }),
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
    build: {
      assetsInlineLimit: 0,
    },
    ssr: {
      external: ["svgo"],
    },
  },
});
