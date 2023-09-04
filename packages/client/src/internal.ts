import { Operation, ProceduresDef, ResponseInner } from ".";
import * as rspc from "./bindings";

export type BatchedItem<P extends ProceduresDef> = {
  op: Operation;
  resolve: (result: P[keyof ProceduresDef]["result"]) => void;
  reject: (error: P[keyof ProceduresDef]["error"] | rspc.Error) => void;
  abort: AbortController;
};

/**
 * @internal
 */
export async function _internal_fireResponse<P extends ProceduresDef>(
  resp: ResponseInner,
  i:
    | BatchedItem<P>
    | {
        resolve: (result: P[keyof ProceduresDef]["result"]) => void;
        reject: (error: P[keyof ProceduresDef]["error"] | rspc.Error) => void;
      }
) {
  if ("abort" in i && i.abort.signal?.aborted) {
    return;
  }

  switch (resp.type) {
    case "value":
      return i.resolve(resp.value);
    case "error":
      if ("Exec" in resp.value) i.reject(resp.value.Exec);
      else i.reject(resp.value.Resolver);
      return;
    case "complete":
      // TODO
      return;
  }
}

// Copied from: https://github.com/jonschlinkert/is-plain-object
function isPlainObject(o: any): o is Object {
  if (!hasObjectPrototype(o)) {
    return false;
  }

  // If has modified constructor
  const ctor = o.constructor;
  if (typeof ctor === "undefined") {
    return true;
  }

  // If has modified prototype
  const prot = ctor.prototype;
  if (!hasObjectPrototype(prot)) {
    return false;
  }

  // If constructor does not have an Object-specific method
  if (!prot.hasOwnProperty("isPrototypeOf")) {
    return false;
  }

  // Most likely a plain Object
  return true;
}

function hasObjectPrototype(o: any): boolean {
  return Object.prototype.toString.call(o) === "[object Object]";
}

// This is copied from the React Query `hashQueryKey` function.
export function hashOperation(queryKey: Omit<Operation, "context">): string {
  return JSON.stringify(queryKey, (_, val) =>
    isPlainObject(val)
      ? Object.keys(val)
          .sort()
          .reduce((result, key) => {
            result[key] = val[key];
            return result;
          }, {} as any)
      : val
  );
}

export function hashedQueryKey(queryKey: Omit<Operation, "context">) {
  const s = hashOperation(queryKey);

  for (var i = 0, h = 9; i < s.length; )
    h = Math.imul(h ^ s.charCodeAt(i++), 9 ** 9);
  return h ^ (h >>> 9);
}
