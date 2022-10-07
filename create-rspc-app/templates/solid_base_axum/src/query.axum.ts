import { QueryClient } from '@tanstack/solid-query';
import { FetchTransport, createClient } from '@rspc/client';
import { createSolidQueryHooks } from '@rspc/solid';

import type { Procedures } from "./bindings"; 

const client = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:9000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createSolidQueryHooks<Procedures>();

export {  client, queryClient}
export default rspc;