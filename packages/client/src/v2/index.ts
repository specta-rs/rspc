// TODO: AbortController's
// TODO: Batching websocket messages
// TODO: How to integrate with Spacedrive's bullshit

// TODO: Middleware & middleware chaining

import { Client, ProceduresDef, Transport } from "..";

export function initRspc<P extends ProceduresDef>(transport: Transport) {
  return new Client<P>({
    transport,
  });
}
