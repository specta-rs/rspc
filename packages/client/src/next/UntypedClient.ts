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

		return await new Promise((res, rej) => {
			obs.subscribe({
				next(value) {
					if (value.type === "data") res(value.value);
				},
				error(value) {
					rej(value);
				},
			});
		});
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

		observable.subscribe({
			next(event) {
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
				}
			},
			error(error) {
				opts?.onError?.(error);
			},
			complete() {
				opts?.onComplete?.();
			},
		});
	}
}
