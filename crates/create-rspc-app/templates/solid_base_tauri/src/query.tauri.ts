import { createClient } from '@rspc/client';
import { QueryClient } from '@tanstack/solid-query';
import { createSolidQueryHooks } from '@rspc/solid';
import { TauriTransport } from '@rspc/tauri';
import type { Procedures } from "./bindings"; // These were the bindings exported from your Rust code!

const client = createClient<Procedures>({
	transport: new TauriTransport()
});

const queryClient = new QueryClient();
const rspc = createSolidQueryHooks<Procedures>();

export { client, queryClient };
export default rspc;