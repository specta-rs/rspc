import { type Channel, invoke } from "@tauri-apps/api/core";

type Request =
  | {
      method: "request";
      params: { path: string; input: null | string };
    }
  | { method: "abort"; params: number };

type Response<T> = { code: number; value: T } | "Done";

// TODO: Seal `Channel` within a standard interface for all "modern links"?
// TODO: handle detect and converting to rspc error class
// TODO: Catch Tauri errors -> Assuming it would happen on `tauri::Error` which happens when serialization fails in Rust.
// TODO: Return closure for cleanup

export async function handleRpc(req: Request, channel: Channel<Response<any>>) {
  await invoke("plugin:rspc|handle_rpc", { req, channel });
}
