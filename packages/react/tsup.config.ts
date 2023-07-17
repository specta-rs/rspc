import { defineConfig } from "tsup";

export default defineConfig((opts) => ({
  entryPoints: ["src/index.tsx"],
  format: ["cjs", "esm"],
  clean: !opts.watch,
  outDir: "dist",
  target: "es2017",
  dts: true,
}));
