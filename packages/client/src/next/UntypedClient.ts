import type {
	ExeceuteData,
	ExecuteArgs,
	ExecuteFn,
	SubscriptionObserver,
} from "./types";

export class UntypedClient {
	constructor(public execute: ExecuteFn) {}

	private async executeAsPromise(args: ExecuteArgs) {
		const obs = this.execute(args);

		let data: ExeceuteData | undefined;

		while (data?.type !== "data" && data?.type !== "error") {
			await obs.subscribe((d) => {
				data = d;
			});
		}

		if (data.type === "error") throw new Error(data.error as any);

		return data.value;
	}

	public query(path: string, input: unknown) {
		return this.executeAsPromise({ type: "query", path, input });
	}
	public mutation(path: string, input: unknown) {
		return this.executeAsPromise({ type: "mutation", path, input });
	}
	public subscription(
		path: string,
		input: unknown,
		opts?: Partial<SubscriptionObserver<unknown, unknown>>,
	) {
		const observable = this.execute({ type: "subscription", path, input });

		observable.subscribe((event) => {
			switch (event.type) {
				case "started": {
					opts?.onStarted?.();
					break;
				}
				case "complete": {
					opts?.onComplete?.();
					break;
				}
				case "data": {
					opts?.onData?.(event.value);
					break;
				}
				case "error": {
					opts?.onError?.(event.error);
					break;
				}
			}
		});
	}
}
