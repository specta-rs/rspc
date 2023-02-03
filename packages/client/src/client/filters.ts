import { ClientArgs } from ".";
import { ProceduresDef } from "../bindings";

export interface FilterData<P extends ProceduresDef> {
  procedureKey:
    | [P["queries" | "mutations" | "subscriptions"]["key"]]
    | [P["queries" | "mutations" | "subscriptions"]["key"], any];
}

export type FilterFn<I extends ProceduresDef> = (
  i: FilterData<I>
) => FilterData<any>;

export type ApplyFilter<
  TProcs extends ProceduresDef,
  TArgs extends ClientArgs<TProcs>
> = TArgs["filter"] extends FilterFn<TProcs>
  ? ReturnType<TArgs["filter"]> extends FilterData<infer P>
    ? P
    : never
  : TProcs;
