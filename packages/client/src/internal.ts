import { Operation, RSPCError, ValueOrError } from ".";

export type BatchedItem = {
  op: Operation;
  resolve: (result: any) => void;
  reject: (error: Error | RSPCError) => void;
  abort: AbortController;
};

export async function fireResponse(
  resp: ValueOrError,
  i:
    | BatchedItem
    | {
        resolve: (result: any) => void;
        reject: (error: Error | RSPCError) => void;
      }
) {
  if ("abort" in i && i.abort.signal?.aborted) {
    return;
  }

  if (resp.type === "value") {
    i.resolve(resp.value);
  } else if (resp.type === "error") {
    i.reject(new RSPCError(resp.value.code, resp.value.message));
  } else {
    console.error("rspc: batch response type mismatch!");
    i.reject(new RSPCError(500, "batch response type mismatch"));
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
