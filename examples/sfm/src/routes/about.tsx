import { Title } from "@solidjs/meta";
import type { RouteDefinition } from "@solidjs/router";

export const route = {
  preload: () => {
    console.log("Preloading data for about");
  },
} satisfies RouteDefinition;

export default function About() {
  return (
    <main>
      <Title>About</Title>
      <h1>About</h1>
    </main>
  );
}
