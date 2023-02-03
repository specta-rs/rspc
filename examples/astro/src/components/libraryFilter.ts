import { ProcedureDef, ProceduresDef } from "@rspc/client/src/bindings";
import {
  FilterData,
  Mutations,
  Queries,
  Subscriptions,
} from "@rspc/client/src";

interface LibraryArgs<T> {
  library_id: string;
  arg: T;
}

type StripLibraryArgsFromInput<T extends ProcedureDef> = T extends any
  ? T["input"] extends LibraryArgs<infer E>
    ? {
        key: T["key"];
        input: E;
        result: T["result"];
      }
    : never
  : never;

type LibraryProcedures<P extends ProcedureDef> = Exclude<
  Extract<P, { input: LibraryArgs<any> }>,
  { input: never }
>;

type AsLibraryProcedure<P extends ProcedureDef> = StripLibraryArgsFromInput<
  LibraryProcedures<P>
>;

type LibraryProceduresFilter<P extends ProceduresDef> = {
  queries: AsLibraryProcedure<Queries<P>>;
  mutations: AsLibraryProcedure<Mutations<P>>;
  subscriptions: AsLibraryProcedure<Subscriptions<P>>;
};

const LIBRARY_ID = "lmaoo";

export const libraryProceduresFilter = <P extends ProceduresDef>(
  p: FilterData<P>
): FilterData<LibraryProceduresFilter<P>> => {
  return {
    ...p,
    procedureKey: [
      p.procedureKey[0],
      { library_id: LIBRARY_ID, args: p.procedureKey[1] ?? null },
    ],
  };
};
