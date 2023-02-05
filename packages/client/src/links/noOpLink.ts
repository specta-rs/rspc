import { Link, ProceduresDef } from "..";

export function noOpLink<
  T extends ProceduresDef,
  TSupportSubscriptions extends boolean = true
>(_?: {
  supportsSubscriptions: TSupportSubscriptions;
}): Link<
  T,
  T,
  {
    terminatedLink: true;
  } & (TSupportSubscriptions extends false
    ? { subscriptionsUnsupported: true }
    : {})
> {
  return undefined as any; // TODO: Working link
}
