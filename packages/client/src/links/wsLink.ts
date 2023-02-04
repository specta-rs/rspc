import { LinkFlag, Link, ProceduresDef } from "..";

// TODO: Allow delaying initialisation of websocket to first connection -> Good for Mattrax's auth flow

export function wsLink<T extends ProceduresDef, TFlag extends LinkFlag>(): Link<
  T,
  T,
  "terminatedLink"
> {
  return undefined as any; // TODO: Working websocket link
}
