import { Channel, invoke } from "@tauri-apps/api/core";
import { ExecuteArgs, ExecuteFn, observable } from "@rspc/client/next";

type Request = { request: { path: string; input: any } } | { abort: number };

type Response<T> = { code: number; value: T } | null;

export async function handleRpc(req: Request, channel: Channel<Response<any>>) {
	await invoke("plugin:rspc|handle_rpc", { req, channel });
}

export const tauriExecute: ExecuteFn = (args: ExecuteArgs) => {
	return observable((subscriber) => {
		const channel = new Channel<Response<any>>();

		channel.onmessage = (response) => {
			if (response === null) {
				subscriber.complete();
				return;
			}
			if (response.code === 200) {
				subscriber.next({ type: "data", value: response.value });
			} else {
				subscriber.error(response.value);
			}
		};

		handleRpc(
			{ request: { path: args.path, input: args.input ?? null } },
			channel,
		)
			.then(() => {
				if (args.type === "subscription") {
					subscriber.next({ type: "started" });
				}
			})
			.catch(() => {
				subscriber.complete();
			});
	});
};
