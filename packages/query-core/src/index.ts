import * as rspc from "@rspc/client";
import type * as tanstack from "@tanstack/query-core";

export * from "./useUtils";

export type WrapQueryOptions<T> = Omit<T, "queryKey" | "queryFn">;
export type WrapMutationOptions<T> = Omit<T, "mutationKey" | "mutationFn">;

export interface SubscriptionOptions<TValue, TError>
	extends Partial<rspc.SubscriptionObserver<TValue, TError>> {
	enabled?: boolean;
}

export interface Context<P extends rspc.Procedures> {
	client: rspc.Client<P>;
	queryClient: tanstack.QueryClient;
}

export function createQueryHooksHelpers<P extends rspc.Procedures>() {
	// hardcoding return types avoids a typescript problem

	function queryHookArgs(
		client: rspc.Client<P>,
		path: string[],
		input: unknown,
		opts?: WrapQueryOptions<tanstack.QueryObserverOptions>,
	): tanstack.QueryObserverOptions {
		return {
			...opts,
			queryKey: rspc.getQueryKey(path.join("."), input),
			queryFn: () =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "query" }
					>(client, path)
					.query(input),
		};
	}

	function mutationHookArgs(
		client: rspc.Client<P>,
		path: string[],
		opts?: WrapMutationOptions<tanstack.MutationObserverOptions>,
	): tanstack.MutationObserverOptions {
		return {
			...opts,
			mutationKey: [path.join(".")],
			mutationFn: (input) =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "mutation" }
					>(client, path)
					.mutate(input),
		};
	}

	function handleSubscription(
		client: rspc.Client<P>,
		path: string[],
		input: unknown,
		opts: () => SubscriptionOptions<unknown, unknown> | undefined,
	): undefined | (() => void) {
		if (!(opts()?.enabled ?? true)) return;

		let isStopped = false;

		const { unsubscribe } = rspc
			.traverseClient<
				Omit<rspc.Procedure, "variant"> & { variant: "subscription" }
			>(client, path)
			.subscribe(input, {
				onStarted: () => !isStopped && opts?.()?.onStarted?.(),
				onData: (data) => !isStopped && opts?.()?.onData?.(data),
				onError: (err) => !isStopped && opts?.()?.onError?.(err),
				onStopped: () => !isStopped && opts?.()?.onStopped?.(),
				onComplete: () => !isStopped && opts?.()?.onComplete?.(),
			});

		return () => {
			isStopped = true;
			unsubscribe?.();
		};
	}

	return { queryHookArgs, mutationHookArgs, handleSubscription };
}
