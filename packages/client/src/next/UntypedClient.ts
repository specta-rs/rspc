import type {
	ExeceuteData,
	ExecuteArgs,
	ExecuteFn,
	SubscriptionObserver,
} from "./types";

export function observable<T>(
	cb: (subscriber: { next: (value: T) => void; complete(): void }) => void,
) {
	let callbacks: Array<(v: T) => void> = [];
	let completeCallbacks: Array<() => void> = [];
	let done = false;

	cb({
		next: (v) => {
			if (done) return;
			callbacks.forEach((cb) => cb(v));
		},
		complete: () => {
			if (done) return;
			done = true;
			completeCallbacks.forEach((cb) => cb());
		},
	});

	return {
		subscribe(cb: (v: T) => void) {
			if (done) return Promise.resolve();

			callbacks.push(cb);
			return new Promise<void>((res) => {
				completeCallbacks.push(() => res());
			});
		},
		get done() {
			return done;
		},
	};
}

export type Observable<T> = ReturnType<typeof observable<T>>;

export const fetchExecute = (
	config: { url: string },
	args: ExecuteArgs,
): ReturnType<ExecuteFn> => {
	if (args.type === "subscription")
		throw new Error("Subscriptions are not possible with the `fetch` executor");

	let promise;
	if (args.type === "query") {
		promise = fetch(
			`${config.url}/${args.path}?${new URLSearchParams({
				input: JSON.stringify(args.input),
			})}`,
			{
				method: "GET",
				headers: {
					Accept: "application/json",
				},
			},
		);
	} else {
		promise = fetch(`${config.url}/${args.path}`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Accept: "application/json",
			},
			body: JSON.stringify(args.input),
		});
	}

	return observable((subscriber) => {
		promise
			.then(async (r) => {
				let json;

				if (r.status === 200) {
					subscriber.next(await r.json());
				} else json = (await r.json()) as [];

				json;
			})
			.finally(() => subscriber.complete());
	});
};

export class UntypedClient {
	constructor(public execute: ExecuteFn) {}

	private async executeAsPromise(args: ExecuteArgs) {
		const obs = this.execute(args);

		let data: ExeceuteData | undefined;

		await obs.subscribe((d) => {
			if (data === undefined) data = d;
		});

		if (!data) throw new Error("No data received");
		if (data.code !== 200)
			throw new Error(`Error with code '${data.code}' occurred`, data.value);

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

		// observable.subscribe((response) => {
		// 	if (response.result.type === "started") {
		// 		opts?.onStarted?.();
		// 	} else if (response.result.type === "stopped") {
		// 		opts?.onStopped?.();
		// 	} else {
		// 		opts?.onData?.(response.result.data);
		// 	}
		// });
	}
}
