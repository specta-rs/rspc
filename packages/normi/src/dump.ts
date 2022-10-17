import { observable, TRPCLink } from "@rspc/client";
import type { QueryClient } from "@tanstack/react-query";

export * from "./hooks";

type NormiCache = Map<string /* $type */, Map<string /* $id */, any>>;

declare global {
  interface Window {
    normiCache?: NormiCache;
  }
}

export interface NormiOptions {
  contextSharing?: boolean;
}

function getNormiCache(contextSharing: boolean): NormiCache {
  if (contextSharing) {
    if (window.normiCache === undefined) {
      window.normiCache = new Map();
    }

    return window.normiCache;
  } else {
    return new Map();
  }
}

function getOrCreate<K, A, B>(map: Map<K, Map<A, B>>, key: K): Map<A, B> {
  let m = map.get(key);
  if (m === undefined) {
    m = new Map();
    map.set(key, m);
  }
  return m;
}

function normaliseValue(value: any, normiCache: NormiCache): any {
  if (value === null || value === undefined) {
    return value;
  } else if (typeof value === "object") {
    if ("$id" in value && "$type" in value) {
      getOrCreate(normiCache, value.$type).set(
        value.$id,
        normaliseValueForStorage(value, true)
      );
      delete value.$id;
      delete value.$type;
    } else if ("$type" in value && "edges" in value) {
      // TODO: Caching all the edges
      value = (value.edges as any[]).map((v) => normaliseValue(v, normiCache));
    }

    // TODO: Optimise this to only check fields the backend marks as normalisable or on root
    for (const [k, v] of Object.entries(value)) {
      value[k] = normaliseValue(v, normiCache);
    }
  }

  return value;
}

function normaliseValueForStorage(value: any, rootElem: boolean): any {
  if (value === null || value === undefined) {
    return value;
  } else if (typeof value === "object") {
    if ("$id" in value && "$type" in value) {
      if (rootElem) {
        let v = Object.assign({}, value);
        delete v.$id;
        delete v.$type;

        // TODO: Optimise this to only check fields the backend marks as normalisable or on root
        for (const [k, vv] of Object.entries(v)) {
          v[k] = normaliseValueForStorage(vv, false);
        }

        return v;
      }

      // TODO: Optimise this to only check fields the backend marks as normalisable or on root
      for (const [k, v] of Object.entries(value)) {
        value[k] = normaliseValueForStorage(v, false);
      }

      return {
        $id: value.$id,
        $type: value.$type,
      };
    } else if ("$type" in value && "edges" in value) {
      return {
        $type: value.$type,
        edges: Object.values(value.edges as any[]).map((v) => v.$id),
      };
    }

    // TODO: Optimise this to only check fields the backend marks as normalisable or on root
    for (const [k, v] of Object.entries(value)) {
      value[k] = normaliseValueForStorage(v, false);
    }
  }

  return value;
}

function recomputeNormalisedValueFromStorage(
  value: any,
  normiCache: NormiCache
): any {
  if (value === null || value === undefined) {
    return value;
  } else if (typeof value === "object") {
    if ("$id" in value && "$type" in value) {
      value = normiCache.get(value.$type)!.get(value.$id); // TODO: Handle `undefined`
    } else if ("$type" in value && "edges" in value) {
      value = (value.edges as any[]).map(
        (id) => normiCache.get(value.$type)!.get(id) // TODO: Handle `undefined`
      );
    }

    // TODO: Optimise this to only check fields the backend marks as normalisable or on root
    for (const [k, v] of Object.entries(value)) {
      value[k] = recomputeNormalisedValueFromStorage(v, normiCache);
    }
  }

  return value;
}

function recomputeRQCache(queryClient: QueryClient, normiCache: NormiCache) {
  let c = queryClient.getQueryCache();

  // c.getAll().forEach((query) => {
  //   const d = query.state.data;
  //   if (Array.isArray(d)) {
  //     queryClient.setQueryData(
  //       query.queryKey,
  //       d.map((f) => {
  //         if (typeof f?.$id == "string" && normyCache.has(f?.$id)) {
  //           return normyCache.get(f.$id);
  //         }
  //         return f;
  //       })
  //     );
  //   }
  // });
}

// export const normiHook: () => void = () => {};

// export const normiLink: (opts: NormiOptions) => TRPCLink<any> = ({
//   queryClient,
//   contextSharing,
// }: NormiOptions) => {
//   let normiCache = getNormiCache(contextSharing ?? false);

//   // window.demo = () => {
//   //   return recomputeNormalisedValueFromStorage(
//   //     normiCache.get("org")!.get("org-1"),
//   //     normiCache
//   //   );
//   // };

//   // window.demo = () => {
//   //   const x = normiCache.get("org")!.get("org-1");
//   //   normiCache.get("org")!.set("org-1", {
//   //     ...x,
//   //     name: "Update from cache!",
//   //   });
//   //   console.log(normiCache);
//   //   recomputeRQCache(queryClient, normiCache);
//   // };

//   // queryClient.getQueryCache().subscribe(({ type, query }) => {
//   //   if (type === "added") {
//   //     console.log("ADDED", query.queryKey, query.state.data);
//   //   } else if (type === "updated") {
//   //     console.log("UPDATE", query.queryKey, query.state.data);

//   //     const d = query.state.data;
//   //     if (Array.isArray(d)) {
//   //       d.forEach((f) => {
//   //         if (typeof f?.$id == "string") normyCache.set(f.$id, f);
//   //       });
//   //     }
//   //   } else if (type === "removed") {
//   //     console.log("REMOVED", query.queryKey, query.state.data);
//   //   }
//   // });

//   return () => {
//     return ({ next, op }) => {
//       return observable((observer) => {
//         const unsubscribe = next(op).subscribe({
//           next(value) {
//             value.result.data = normaliseValue(value.result.data, normiCache);
//             observer.next(value);
//           },
//           error(err) {
//             observer.error(err);
//           },
//           complete() {
//             observer.complete();
//           },
//         });
//         return unsubscribe;
//       });
//     };
//   };
// };
