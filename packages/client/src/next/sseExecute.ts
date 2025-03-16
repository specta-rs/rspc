import { observable } from "./observable";
import { ExeceuteData, ExecuteArgs, ExecuteFn } from "./types";

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
	const fullUrl = `${sseArgs.url}/${args.path}`;

	const sse = sseArgs.makeEventSource
		? sseArgs.makeEventSource(fullUrl, sseArgs.eventSourceInitDict)
		: new EventSource(fullUrl, sseArgs.eventSourceInitDict);

	return observable<ExeceuteData, any>((o) => {
		sse.onopen = () => {
			o.next({ type: "started" });
		};
		sse.onmessage = (e) => {
			if (e.data === "stopped") {
				sse.close();
				o.complete();
				return;
			}

			const value:
				| { item: any }
				| {
						error: { code: number; message: String; data: any };
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
