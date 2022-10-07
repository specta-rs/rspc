import { createClient } from '@rspc/client';
import { createReactQueryHooks } from '@rspc/react';
import { TauriTransport } from '@rspc/tauri';
import { QueryClient } from '@tanstack/react-query';
import type { Procedures } from "./bindings"; // These were the bindings exported from your Rust code!

const client = createClient<Procedures>({
	transport: new TauriTransport()
});

const queryClient = new QueryClient();
const rspc = createReactQueryHooks<Procedures>();

export {client, queryClient}
export default rspc;