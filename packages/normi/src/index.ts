// TODO: Subscribing to backend alerts for changes
// TODO: Modify data in cache
// TODO: Fixing types

import { TRPCLink, DataTransformer } from "@rspc/client";
import type { QueryClient } from "@tanstack/react-query";

export interface NormiOptions {
  queryClient: QueryClient;
}

export function normi({ queryClient }: NormiOptions): DataTransformer {
  return {
    serialize(data) {
      console.log("NORMI IN", data);
      return data;
    },
    deserialize(data) {
      console.log("NORMI OUT", data);
      return data;
    },
  };
}
