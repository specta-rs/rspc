import { Channel, invoke } from "@tauri-apps/api/core";

type Request =
	| {
			method: "request";
			params: { path: string; input: null | string };
	  }
	| { method: "abort"; params: number };

type Response<T> = { Value: { code: number; value: T } } | "Done";

export async function handleRpc(req: Request, channel: Channel<Response<any>>) {
	await invoke("plugin:rspc|handle_rpc", { req, channel });
}
