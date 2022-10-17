const withSolid = require("rollup-preset-solid").default;

export default withSolid({
  targets: ["esm", "cjs"],
});
