import githubIconLight from "./assets/github-light.png";
import githubIconDark from "./assets/github-dark.png";
import discordIcon from "./assets/discord.png";
import npmIcon from "./assets/npm.png";
import cargoIcon from "./assets/cargo.png";

export interface Config {
  repository: string;
  seo: {
    title: string;
    description?: string;
    author?: string;
    keywords?: string[];
  };
  header: {
    links: {
      alt: string;
      href: string;
      src: string; // This could be URL or base64 inlined version
      darkSrc?: string;
    }[];
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
        src: cargoIcon,
      },
      {
        alt: "npm",
        href: "https://www.npmjs.com/org/rspc",
        src: npmIcon,
      },
      {
        alt: "Discord",
        href: "https://discord.gg/4V9M5sksw8",
        src: discordIcon,
      },
      {
        alt: "GitHub",
        href: "https://github.com/oscartbeaumont/rspc",
        src: githubIconLight,
        darkSrc: githubIconDark,
      },
    ],
  },
};
