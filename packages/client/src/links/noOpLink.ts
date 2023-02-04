import { LinkFlag, Link, ProceduresDef } from "..";

export function noOpLink<
  T extends ProceduresDef,
  TFlag extends LinkFlag,
  TSupportSubscriptions extends boolean = true
>(opts?: {
  supportsSubscriptions: TSupportSubscriptions;
}): Link<
  T,
  T,
  TSupportSubscriptions extends false
    ? "terminatedLink" | "subscriptionsUnsupported"
    : "terminatedLink"
> {
  return undefined as any; // TODO: Working websocket link
}
