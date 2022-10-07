import { QueryClient } from '@tanstack/react-query';
import { FetchTransport, createClient } from '@rspc/client';
import { createReactQueryHooks } from '@rspc/react';

import type { Procedures } from "./bindings"; // These were the bindings exported from your Rust code!

// You must provide the generated types as a generic and create a transport (in this example we are using HTTP Fetch) so that the client knows how to communicate with your API.
const client = createClient<Procedures>({
  // Refer to the integration your using for the correct transport.
  transport: new FetchTransport("http://localhost:4000/rspc"),
});

const queryClient = new QueryClient();
const rspc = createReactQueryHooks<Procedures>();

export {client, queryClient}
export default rspc;