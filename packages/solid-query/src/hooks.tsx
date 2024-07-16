import * as rspc from "@rspc/client";
import * as tanstack from "@tanstack/solid-query";
import * as solid from "solid-js";

import type { Context, ProviderProps, SolidQueryProxyBuiltins } from "./types";

export function createHooks<P extends rspc.Procedures>() {
	const context = solid.createContext<Context<P>>();

	function useContext() {
		const ctx = solid.useContext(context);
		if (!ctx)
			throw new Error(
				"The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree.",
			);
		return ctx;
	}

	function createQuery(
		path: string[],
		...[input, opts]: [
			solid.Accessor<unknown>,
			(
				| solid.Accessor<
						tanstack.CreateQueryOptions<unknown, unknown, unknown, any>
				  >
				| undefined
			),
		]
	) {
		const ctx = useContext();
		const client = ctx.client;

		const pathString = path.join(".");

		return tanstack.createQuery(() => ({
			...opts?.(),
			queryKey: rspc.getQueryKey(pathString, input()),
			queryFn: () =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "query" }
					>(client, path)
					.query(input()),
		}));
	}

	function createMutation(
		path: string[],
		...[opts]: [
			| solid.Accessor<
					tanstack.SolidMutationOptions<unknown, unknown, unknown, unknown>
			  >
			| undefined,
		]
	) {
		const ctx = useContext();
		const client = ctx.client;

		return tanstack.createMutation(() => ({
			...opts?.(),
			mutationKey: [path],
			mutationFn: (input) =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "mutation" }
					>(client, path)
					.mutate(input),
		}));
	}

	function createSubscription(
		path: string[],
		...[input, opts]: [
			solid.Accessor<unknown>,
			opts:
				| solid.Accessor<Partial<rspc.SubscriptionObserver<unknown, unknown>>>
				| undefined,
		]
	) {
		const enabled = () => /* opts?.().enabled ?? */ true;
		const ctx = useContext();
		const client = ctx.client;

		solid.createEffect(
			solid.on(
				() => [input(), enabled()],
				([input, enabled]) => {
					if (!enabled) return;

					let isStopped = false;

					const { unsubscribe } = rspc
						.traverseClient<
							Omit<rspc.Procedure, "variant"> & { variant: "subscription" }
						>(client, path)
						.subscribe(input, {
							onStarted: () => {
								if (!isStopped) opts?.().onStarted?.();
							},
							onData: (data) => {
								if (!isStopped) opts?.().onData?.(data);
							},
							onError: (err) => {
								if (!isStopped) opts?.().onError?.(err);
							},
							onStopped: () => {
								if (!isStopped) opts?.().onStopped?.();
							},
							onComplete: () => {
								if (!isStopped) opts?.().onComplete?.();
							},
						});

					solid.onCleanup(() => {
						isStopped = true;
						unsubscribe?.();
					});
				},
			),
		);
	}

	function Provider(props: ProviderProps<P>) {
		return (
			<context.Provider
				value={{ client: props.client, queryClient: props.queryClient }}
			>
				{props.children}
			</context.Provider>
		);
	}

	const hooks = {
		Provider,
	} satisfies SolidQueryProxyBuiltins<P>;

	return Object.assign(hooks, {
		createQuery,
		createMutation,
		createSubscription,
	});
}
