import { observable } from "./observable";
import type { ExeceuteData, ExecuteArgs, ExecuteFn } from "./types";

interface SSEExecuteArgs {
	url: string;
	eventSourceInitDict?: EventSourceInit;
	makeEventSource?: (
		url: string,
		eventSourceInitDict?: EventSourceInit,
	) => EventSource;
}

export function sseExecute(
	sseArgs: SSEExecuteArgs,
	args: ExecuteArgs,
): ReturnType<ExecuteFn> {
	let fullUrl = `${sseArgs.url}/${args.path}`;
	if (args.input !== undefined) {
		const encodedInput = encodeURIComponent(JSON.stringify(args.input));
		fullUrl += `?input=${encodedInput}`;
	}

	const sse = sseArgs.makeEventSource
		? sseArgs.makeEventSource(fullUrl, sseArgs.eventSourceInitDict)
		: new EventSource(fullUrl, sseArgs.eventSourceInitDict);

	return observable<ExeceuteData, any>((o) => {
		sse.onopen = () => {
			o.next({ type: "started" });
		};
		sse.onmessage = (e) => {
			console.log("message", e);
			if (e.data === "stopped") {
				sse.close();
				o.complete();
				return;
			}

			const value:
				| { item: any }
				| {
						error: { code: number; message: string; data: any };
				  } = JSON.parse(e.data);

			if ("item" in value) {
				o.next({ type: "data", value: value.item });
			} else if ("error" in value) {
				o.error(value.error.data);
				sse.close();
			}
		};
	});
}
