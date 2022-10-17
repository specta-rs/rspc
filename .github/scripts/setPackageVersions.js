/**
 * Updates the package.json files with a version to release to npm under
 * the main tag.
 *
 * Based on https://github.com/facebook/relay/blob/main/gulpfile.js
 */

const fs = require("fs/promises");
const path = require("path");

const RELEASE_COMMIT_SHA = process.env.RELEASE_COMMIT_SHA;

if (RELEASE_COMMIT_SHA && RELEASE_COMMIT_SHA.length !== 40) {
  throw new Error(
    "If the RELEASE_COMMIT_SHA env variable is set, it should be set to the " +
      "40 character git commit hash."
  );
}

const VERSION = RELEASE_COMMIT_SHA
  ? `0.0.0-main-${RELEASE_COMMIT_SHA.substring(0, 8)}`
  : process.env.npm_package_version;

console.log(RELEASE_COMMIT_SHA);

async function main() {
  const packages = await (
    await fs.readdir(path.join(__dirname, "../../packages"))
  ).filter((pkg) => pkg !== ".DS_Store" && pkg !== "tsconfig.json");
  const pkgJsons = {};
  const pkgJsonPaths = {};

  for (pkg of packages) {
    const pkgJsonPath = path.join(
      __dirname,
      "../../packages",
      pkg,
      "package.json"
    );
    pkgJsonPaths[pkg] = pkgJsonPath;
    const packageJson = JSON.parse(await fs.readFile(pkgJsonPath, "utf8"));

    pkgJsons[pkg] = packageJson;
  }

  const packageNames = Object.values(pkgJsons).map((pkg) => pkg.name);

  for (const pkg of packages) {
    let packageJson = pkgJsons[pkg];
    packageJson.version = VERSION;
    for (const depKind of [
      "dependencies",
      "devDependencies",
      "peerDependencies",
    ]) {
      const deps = packageJson[depKind];
      for (const dep in deps) {
        if (packageNames.includes(dep)) {
          deps[dep] = VERSION;
        }
      }
    }
    await fs.writeFile(
      pkgJsonPaths[pkg],
      JSON.stringify(packageJson, null, 2) + "\n",
      "utf8"
    );
  }
}

main();
