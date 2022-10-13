import { TRPCClientRuntime } from "..";

type AnyRouter = any;
type TRPCResponse<A> = any;
type TRPCResponseMessage<A> = any;
type TRPCResultMessage<A> = any;

// FIXME:
// - the generics here are probably unnecessary
// - the RPC-spec could probably be simplified to combine HTTP + WS
/** @internal */
export function transformResult<TRouter extends AnyRouter, TOutput>(
  response: TRPCResponseMessage<TOutput> | TRPCResponse<TOutput>,
  runtime: TRPCClientRuntime
) {
  if (response.result.type === "error") {
    const error = runtime.transformer.deserialize(response.result.data) as any;
    return {
      ok: false,
      error: {
        ...response,
        error,
      },
    } as const;
  }

  const result = {
    ...response.result,
    ...((!response.result.type || response.result.type === "data") && {
      type: "data",
      data: runtime.transformer.deserialize(response.result.data),
    }),
  } as TRPCResultMessage<TOutput>["result"];
  return { ok: true, result } as const;
}
