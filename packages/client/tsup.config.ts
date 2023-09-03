import { defineConfig } from "tsup";

export default defineConfig(() => ({
  entryPoints: ["src/index.ts"],
  format: ["cjs", "esm"],
  outDir: "dist",
  target: "es2017",
}));
