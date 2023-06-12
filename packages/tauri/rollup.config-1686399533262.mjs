import { buildConfig } from '@rspc/config/getRollupConfig';

var rollup_config = buildConfig("src/index.ts");

export { rollup_config as default };
