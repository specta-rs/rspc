import type { ProcedureKind } from "./types";

export interface SubscriptionObserver<TValue, TError> {
	onStarted: () => void;
	onData: (value: TValue) => void;
	onError: (err: TError) => void;
	onStopped: () => void;
	onComplete: () => void;
}

export class UntypedClient {
	private $request(args: {
		type: ProcedureKind;
		input: unknown;
		path: string;
	}) {
		console.log(args);
	}

	private $requestAsPromise(args: {
		type: ProcedureKind;
		path: string;
		input: unknown;
	}) {
		return this.$request(args);
	}

	public query(path: string, input: unknown) {
		return this.$requestAsPromise({ type: "query", path, input });
	}
	public mutation(path: string, input: unknown) {
		return this.$requestAsPromise({ type: "mutation", path, input });
	}
	public subscription(
		path: string,
		input: unknown,
		opts?: Partial<SubscriptionObserver<unknown, unknown>>,
	) {
		const observable = this.$request({ type: "subscription", path, input });

		return { unsubscribe: () => {} };
		// observable.subscribe({
		// 	next(envelope) {
		// 		if (envelope.result.type === "started") {
		// 			opts?.onStarted?.();
		// 		} else if (envelope.result.type === "stopped") {
		// 			opts?.onStopped?.();
		// 		} else {
		// 			opts?.onData?.(envelope.result.data);
		// 		}
		// 	},
		// });
	}
}
