import { ProceduresLike } from "..";
import {
  TRPCClient as Client,
  CreateTRPCClientOptions,
} from "./internals/TRPCClient";
// import { httpBatchLink } from "./links";

export function createClient<TProcedures extends ProceduresLike>(
  opts: CreateTRPCClientOptions<TProcedures>
) {
  const getLinks = () => {
    if ("links" in opts) {
      return opts.links;
    }
    // return [httpBatchLink(opts)];
    throw new Error("BRUH");
  };
  const client = new Client<TProcedures>({
    transformer: opts.transformer,
    links: getLinks(),
  });
  return client;
}

export type {
  CreateTRPCClientOptions,
  TRPCClient,
  TRPCRequestOptions,
} from "./internals/TRPCClient";
