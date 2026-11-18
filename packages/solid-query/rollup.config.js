import withSolid from "rollup-preset-solid";

export default withSolid({
  targets: ["esm", "cjs"],
  // `rollup-preset-solid` still ships `babel-preset-solid` (the Solid 1 compiler), so
  // override its presets with Solid 2's Babel plugin. We lose `solidOptions` in the
  // process, but the Solid 2 defaults are what we want here anyway.
  babelOptions: {
    presets: [
      "@babel/preset-typescript",
      ["@babel/preset-env", { bugfixes: true, targets: "last 2 years" }],
    ],
    plugins: [["@solidjs/babel-plugin", {}]],
  },
});
