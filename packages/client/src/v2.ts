// TODO: AbortController's
// TODO: Batching websocket messages
// TODO: How to integrate with Spacedrive's bullshit

// TODO: Middleware & middleware chaining

import { ProceduresDef, Transport } from ".";
import { AlphaClient } from "./v2/client";
export * from "./v2/client";

export function initRspc<P extends ProceduresDef>(transport: Transport) {
  return new AlphaClient<P>({
    transport,
  });
}
