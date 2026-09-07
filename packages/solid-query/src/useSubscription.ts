import { createEffect } from "solid-js";
import { createStore } from "solid-js/store";
import type { RspcSubscriptionOptions } from "./createOptionsProxy";

export type SubscriptionStatus = "idle" | "pending" | "success" | "error";

export type SubscriptionResult<TOut, TError> = {
	data: TOut | undefined;
	error: TError | undefined;
	status: SubscriptionStatus;
};

export function useSubscription<TOut, TError>(
	options: RspcSubscriptionOptions<TOut, TError>,
) {
	const [state, setState] = createStore<SubscriptionResult<TOut, TError>>({
		data: undefined,
		error: undefined,
		status: "idle",
	});

	createEffect(() => {
		if (!options.enabled) {
			return;
		}

		options.subscribe({
			onStarted() {
				options.onStarted?.();
				setState({
					data: undefined,
					error: undefined,
					status: "pending",
				});
			},
			onData(value) {
				options.onData?.(value);
				setState({
					data: value,
					error: undefined,
					status: "pending",
				});
			},
			onComplete() {
				options.onComplete?.();
				setState((state) => ({
					...state,
					status: "success",
				}));
			},
			onError(error) {
				options.onError?.(error);
				setState((state) => ({
					...state,
					error,
					status: "error",
				}));
			},
		});
	});

	return state;
}
