import { defineConfig } from "tsup";

export default defineConfig(() => ({
  entryPoints: ["src/index.tsx"],
  format: ["cjs", "esm"],
}));
