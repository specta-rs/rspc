import * as rspc from "@rspc/client";
import * as tanstack from "@tanstack/react-query";
import * as react from "react";

import type { Context, ProviderProps, ReactQueryProxyBuiltins } from "./types";

export function createHooks<P extends rspc.Procedures>() {
	const context = react.createContext<Context<P> | undefined>(undefined);

	function useContext() {
		const ctx = react.useContext(context);
		if (!ctx)
			throw new Error(
				"The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree.",
			);
		return ctx;
	}

	function useQuery(
		path: string[],
		...[input, opts]: [
			unknown,
			(
				| Omit<
						tanstack.UseQueryOptions<unknown, unknown, unknown, any>,
						"queryKey" | "queryFn"
				  >
				| undefined
			),
		]
	) {
		const ctx = useContext();
		const client = ctx.client;

		const pathString = path.join(".");

		return tanstack.useQuery({
			...opts,
			queryKey: rspc.getQueryKey(pathString, input),
			queryFn: () =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "query" }
					>(client, path)
					.query(input),
		});
	}

	function useMutation(
		path: string[],
		...[opts]: [
			tanstack.UseMutationOptions<unknown, unknown, unknown, unknown>,
			undefined,
		]
	) {
		const ctx = useContext();
		const client = ctx.client;

		return tanstack.useMutation({
			...opts,
			mutationKey: [path],
			mutationFn: (input) =>
				rspc
					.traverseClient<
						Omit<rspc.Procedure, "variant"> & { variant: "mutation" }
					>(client, path)
					.mutate(input),
		});
	}

	function useSubscription(
		path: string[],
		...[input, opts]: [
			unknown,
			Partial<rspc.SubscriptionObserver<unknown, unknown>>,
		]
	) {
		const enabled = /*opts?.enabled ??*/ true;

		const ctx = useContext();
		const client = ctx.client;

		const queryKey = tanstack.hashKey([...path, input]);

		const optsRef = react.useRef<typeof opts>(opts);
		optsRef.current = opts;

		// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
		react.useEffect(() => {
			if (!enabled) return;

			let isStopped = false;

			const { unsubscribe } = rspc
				.traverseClient<
					Omit<rspc.Procedure, "variant"> & { variant: "subscription" }
				>(client, path)
				.subscribe(input, {
					onStarted: () => {
						if (!isStopped) optsRef.current.onStarted?.();
					},
					onData: (data) => {
						if (!isStopped) optsRef.current.onData?.(data);
					},
					onError: (err) => {
						if (!isStopped) optsRef.current.onError?.(err);
					},
					onStopped: () => {
						if (!isStopped) optsRef.current.onStopped?.();
					},
					onComplete: () => {
						if (!isStopped) optsRef.current.onComplete?.();
					},
				});

			return () => {
				isStopped = true;
				unsubscribe?.();
			};
		}, [queryKey, enabled]);
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
	} satisfies ReactQueryProxyBuiltins<P>;

	return Object.assign(hooks, { useQuery, useMutation, useSubscription });
}
