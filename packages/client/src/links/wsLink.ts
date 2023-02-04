import { Link, ProceduresDef } from "..";

// TODO: Allow delaying initialisation of websocket to first connection -> Good for Mattrax's auth flow

export function wsLink<T extends ProceduresDef>(): Link<
  T,
  T,
  { terminatedLink: true }
> {
  return undefined as any; // TODO: Working websocket link
}
