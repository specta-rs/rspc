import { RollupOptions } from "rollup";
import { swc, defineRollupSwcOption } from "rollup-plugin-swc3";
import del from "rollup-plugin-delete";
import typescript from "rollup-plugin-typescript2";
import dtsRaw from "rollup-plugin-dts";
import { visualizer } from "rollup-plugin-visualizer";
import externals from "rollup-plugin-node-externals";
// @ts-expect-error no typedefs exist for this plugin
import multiInput from "rollup-plugin-multi-input";

const dts = (
  typeof dtsRaw !== "function" ? (dtsRaw as any).default : dtsRaw
) as typeof dtsRaw;

const isWatchMode = process.argv.includes("--watch");

export function buildConfig(input: string | string[]): RollupOptions[] {
  return [
    {
      input,
      output: [
        {
          dir: `./dist`,
          format: "cjs",
          entryFileNames: "[name].js",
          chunkFileNames: "[name]-[hash].js",
        },
        {
          dir: `./dist`,
          format: "esm",
          entryFileNames: "[name].mjs",
          chunkFileNames: "[name]-[hash].mjs",
        },
      ],
      plugins: [
        !isWatchMode &&
          del({
            targets: "./dist/*.{js,mjs}",
          }),
        typescript({
          abortOnError: !isWatchMode,
        }),
        externals({
          deps: true,
          devDeps: true,
          peerDeps: true,
        }),
        swc(
          defineRollupSwcOption({
            tsconfig: false,
            sourceMaps: true,
            jsc: {
              target: "es2020",
              transform: {
                react: {
                  useBuiltins: true,
                },
              },
              externalHelpers: true,
            },
          })
        ),
        visualizer({
          gzipSize: true,
          brotliSize: true,
          // TODO: Support for viewing the bundle size of `@rspc/client/full`
        }),
      ],
    },
    {
      input,
      output: { dir: `./dist`, format: "es", entryFileNames: "[name].d.ts" },
      plugins: [
        !isWatchMode &&
          del({
            targets: "./dist/*.d.ts",
          }),
        multiInput.default({ relative: "src/" }),
        dts({
          tsconfig: "tsconfig.json",
          compilerOptions: {
            preserveSymlinks: false,
          },
        }),
      ],
    },
  ];
}
