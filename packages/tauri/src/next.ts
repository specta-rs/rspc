import { Channel, invoke } from "@tauri-apps/api/core";
import { ExecuteArgs, ExecuteFn, observable } from "@rspc/client/next";

type Request =
	| { method: "request"; params: { path: string; input: any } }
	| { method: "abort"; params: number };

type Response<T> = { code: number; value: T } | null;

// TODO: Seal `Channel` within a standard interface for all "modern links"?
// TODO: handle detect and converting to rspc error class
// TODO: Catch Tauri errors -> Assuming it would happen on `tauri::Error` which happens when serialization fails in Rust.
// TODO: Return closure for cleanup

export async function handleRpc(req: Request, channel: Channel<Response<any>>) {
	await invoke("plugin:rspc|handle_rpc", { req, channel });
}

export const tauriExecute: ExecuteFn = (args: ExecuteArgs) => {
	return observable((subscriber) => {
		const channel = new Channel<Response<any>>();

		channel.onmessage = (response) => {
			if (response === null) subscriber.complete();
			return subscriber.next(response);
		};

		handleRpc(
			{
				method: "request",
				params: {
					path: args.path,
					input:
						args.input === undefined || args.input === null ? null : args.input,
				},
			},
			channel,
		);
	});
};
