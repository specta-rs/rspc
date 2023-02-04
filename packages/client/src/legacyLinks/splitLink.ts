import { observable } from "../internals/observable/index";
import { ProceduresDef } from "..";
import { createChain } from "./internals/createChain";
import { LegacyOperation, TRPCLink } from "./types";

function asArray<TType>(value: TType | TType[]) {
  return Array.isArray(value) ? value : [value];
}
export function splitLink<
  TProcedures extends ProceduresDef = ProceduresDef
>(opts: {
  condition: (op: LegacyOperation) => boolean;
  /**
   * The link to execute next if the test function returns `true`.
   */
  true: TRPCLink<TProcedures> | TRPCLink<TProcedures>[];
  /**
   * The link to execute next if the test function returns `false`.
   */
  false: TRPCLink<TProcedures> | TRPCLink<TProcedures>[];
}): TRPCLink<TProcedures> {
  return (runtime) => {
    const yes = asArray(opts.true).map((link) => link(runtime));
    const no = asArray(opts.false).map((link) => link(runtime));
    return (props) => {
      return observable((observer) => {
        const links = opts.condition(props.op) ? yes : no;
        return createChain({ op: props.op, links }).subscribe(observer);
      });
    };
  };
}
