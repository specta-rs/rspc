// TODO: Remove this in the future

import { Procedures } from "../../../bindings";

function createProxy<T>(): { [K in keyof T]: () => T[K] } {
  return undefined as any;
}

const procedures = createProxy<Procedures>();

procedures.version();

procedures.newstuff();
