const withSolid = require("rollup-preset-solid").default;

module.exports = withSolid({
  targets: ["esm", "cjs"],
});
