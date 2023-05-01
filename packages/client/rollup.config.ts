import { buildConfig } from "@rspc/config/getRollupConfig";

export default buildConfig(["src/index.ts", "src/full.ts", "src/v2/index.ts"]);
