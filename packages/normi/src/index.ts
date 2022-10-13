import { observable, TRPCLink } from "@rspc/client";
import type { QueryClient } from "@tanstack/react-query";

type NormiCache = Map<string /* __type */, Map<string /* __id */, any>>;
declare global {
  interface Window {
    normiCache?: NormiCache;
  }
}

export interface NormiOptions {
  queryClient: QueryClient;
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

// function refreshRQ(queryClient: QueryClient) {
//   let c = queryClient.getQueryCache();

//   c.getAll().forEach((query) => {
//     const d = query.state.data;
//     if (Array.isArray(d)) {
//       queryClient.setQueryData(
//         query.queryKey,
//         d.map((f) => {
//           if (typeof f?.__id == "string" && normyCache.has(f?.__id)) {
//             return normyCache.get(f.__id);
//           }
//           return f;
//         })
//       );
//     }
//   });
// }

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
    if ("__id" in value && "__type" in value) {
      getOrCreate(normiCache, value.__type).set(
        value.__id,
        normaliseValueForStorage(value, true)
      );
      delete value.__id;
      delete value.__type;
    } else if ("__type" in value && "edges" in value) {
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
    if ("__id" in value && "__type" in value) {
      if (rootElem) {
        let v = Object.assign({}, value);
        delete v.__id;
        delete v.__type;

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
        __id: value.__id,
        __type: value.__type,
      };
    } else if ("__type" in value && "edges" in value) {
      return {
        __type: value.__type,
        edges: Object.values(value.edges as any[]).map((v) => v.__id),
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
    if ("__id" in value && "__type" in value) {
      value = normiCache.get(value.__type)!.get(value.__id); // TODO: Handle `undefined`
    } else if ("__type" in value && "edges" in value) {
      value = (value.edges as any[]).map(
        (id) => normiCache.get(value.__type)!.get(id) // TODO: Handle `undefined`
      );
    }

    // TODO: Optimise this to only check fields the backend marks as normalisable or on root
    for (const [k, v] of Object.entries(value)) {
      value[k] = recomputeNormalisedValueFromStorage(v, normiCache);
    }
  }

  return value;
}

export const normiLink: (opts: NormiOptions) => TRPCLink<any> = ({
  queryClient,
  contextSharing,
}: NormiOptions) => {
  let normiCache = getNormiCache(contextSharing ?? false);

  // TODO: Disable staleness in React Query
  // TODO: Subscribing to backend alerts for changes
  // TODO: Modify data in cache

  // window.demo = () => {
  //   return recomputeNormalisedValueFromStorage(
  //     normiCache.get("org")!.get("org-1"),
  //     normiCache
  //   );
  // };

  // queryClient.getQueryCache().subscribe(({ type, query }) => {
  //   if (type === "added") {
  //     console.log("ADDED", query.queryKey, query.state.data);
  //   } else if (type === "updated") {
  //     console.log("UPDATE", query.queryKey, query.state.data);

  //     const d = query.state.data;
  //     if (Array.isArray(d)) {
  //       d.forEach((f) => {
  //         if (typeof f?.__id == "string") normyCache.set(f.__id, f);
  //       });
  //     }
  //   } else if (type === "removed") {
  //     console.log("REMOVED", query.queryKey, query.state.data);
  //   }
  // });

  return () => {
    return ({ next, op }) => {
      return observable((observer) => {
        const unsubscribe = next(op).subscribe({
          next(value) {
            value.result.data = normaliseValue(value.result.data, normiCache);
            observer.next(value);
          },
          error(err) {
            observer.error(err);
          },
          complete() {
            observer.complete();
          },
        });
        return unsubscribe;
      });
    };
  };
};
