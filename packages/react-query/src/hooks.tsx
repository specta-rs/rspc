import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/react-query";
import * as react from "react";

import type { ProviderProps, ReactQueryProxyBuiltins } from "./types";

export function createHooks<P extends rspc.Procedures>() {
	const context = react.createContext<queryCore.Context<P> | undefined>(
		undefined,
	);

	const helpers = queryCore.createQueryHooksHelpers<P>();

	function useContext() {
		const ctx = react.useContext(context);
		if (!ctx)
			throw new Error(
				"The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree.",
			);
		return ctx;
	}

	function useClient() {
		return useContext().client;
	}

	function useQuery(
		path: string[],
		...[input, opts]: [
			unknown | tanstack.SkipToken,
			queryCore.WrapQueryOptions<tanstack.UseQueryOptions> | undefined,
		]
	) {
		const client = useClient();
		return tanstack.useQuery(helpers.queryHookArgs(client, path, input, opts));
	}

	function useMutation(
		path: string[],
		...[opts]: [tanstack.UseMutationOptions | undefined]
	) {
		const client = useClient();
		return tanstack.useMutation(helpers.mutationHookArgs(client, path, opts));
	}

	function useSubscription(
		path: string[],
		...[input, opts]: [
			unknown,
			queryCore.SubscriptionOptions<unknown, unknown> | undefined,
		]
	) {
		// trpc does this
		const optsRef = react.useRef<typeof opts>(opts);
		optsRef.current = opts;

		const client = useClient();
		const queryKey = tanstack.hashKey([...path, input]);
		const enabled = optsRef.current?.enabled ?? true;

		// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
		react.useEffect(
			() =>
				helpers.handleSubscription(client, path, input, () => optsRef.current),
			[queryKey, enabled],
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
		useUtils: () => queryCore.createQueryUtilsProxy(useContext()),
	} satisfies ReactQueryProxyBuiltins<P>;

	return Object.assign(hooks, { useQuery, useMutation, useSubscription });
}
