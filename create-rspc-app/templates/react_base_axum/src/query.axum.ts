import { QueryClient } from '@tanstack/react-query';
import { FetchTransport, createClient } from '@rspc/client';
import { createReactQueryHooks } from '@rspc/react';

import type { Procedures } from "./bindings"; 

const client = createClient<Procedures>({
  transport: new FetchTransport("http://localhost:9000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createReactQueryHooks<Procedures>();

export {client, queryClient}
export default rspc;