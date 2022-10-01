import { JSX } from "solid-js";

export interface Config {
  repository: string;
  seo: {
    title: string;
    description?: string;
    author?: string;
    keywords?: string[];
  };
  header: {
    links: ({
      alt: string;
      href: string;
    } & (
      | {
          customIcon: (props: {
            className?: string;
            width?: string;
            height?: string;
          }) => JSX.Element;
        }
      | { icon: string }
    ))[];
  };
}

export const config: Config = {
  // Set the sites default URL in the `astro.config.mjs`.
  repository: "https://github.com/oscartbeaumont/rspc",
  seo: {
    title: "rspc",
    description:
      "The best way to build typesafe APIs between Rust and Typescript.",
    author: "Oscar Beaumont",
    keywords: [],
  },
  header: {
    links: [
      {
        alt: "crates.io",
        href: "https://crates.io/crates/rspc",
        customIcon: ({ className, width, height }) => (
          <svg
            aria-hidden="true"
            // @ts-expect-error
            focusable="false"
            data-prefix="fas"
            data-icon="crates.io logo"
            class={className}
            role="img"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 512 512"
            height={height}
            width={width}
          >
            <path d="M239.1 6.3l-208 78c-18.7 7-31.1 25-31.1 45v225.1c0 18.2 10.3 34.8 26.5 42.9l208 104c13.5 6.8 29.4 6.8 42.9 0l208-104c16.3-8.1 26.5-24.8 26.5-42.9V129.3c0-20-12.4-37.9-31.1-44.9l-208-78C262 2.2 250 2.2 239.1 6.3zM256 68.4l192 72v1.1l-192 78-192-78v-1.1l192-72zm32 356V275.5l160-65v133.9l-160 80z"></path>
          </svg>
        ),
      },
      {
        alt: "npm",
        href: "https://www.npmjs.com/org/rspc",
        icon: "logos:npm-icon",
      },
      {
        alt: "Discord",
        href: "https://discord.gg/4V9M5sksw8",
        icon: "logos:discord-icon",
      },
      {
        alt: "GitHub",
        href: "https://github.com/oscartbeaumont/rspc",
        icon: "fa:github",
      },
    ],
  },
};
