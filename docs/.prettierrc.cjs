// @ts-check

/** @type {import("prettier").Config & import("prettier-plugin-tailwindcss").PluginOptions} */
const prettierConfig = {
  trailingComma: "all",
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  bracketSpacing: true,
  printWidth: 80,
  endOfLine: "lf",
  plugins: [
    require.resolve("prettier-plugin-tailwindcss"),
    require.resolve("prettier-plugin-organize-imports"),
  ],
  tailwindConfig: "./tailwind.config.ts",
};

module.exports = prettierConfig;
