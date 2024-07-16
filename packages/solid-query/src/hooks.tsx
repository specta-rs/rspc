import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/solid-query";
import * as solid from "solid-js";

import type { ProviderProps, SolidQueryProxyBuiltins } from "./types";

export function createHooks<P extends rspc.Procedures>() {
	const context = solid.createContext<queryCore.Context<P>>();

	const helpers = queryCore.createQueryHooksHelpers<P>();

	function useContext() {
		const ctx = solid.useContext(context);
		if (!ctx)
			throw new Error(
				"The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree.",
			);
		return ctx;
	}

	function useClient() {
		return useContext().client;
	}

	function createQuery(
		path: string[],
		...[input, opts]: [
			solid.Accessor<unknown | tanstack.SkipToken>,
			(
				| solid.Accessor<queryCore.WrapQueryOptions<tanstack.SolidQueryOptions>>
				| undefined
			),
		]
	) {
		const client = useClient();
		return tanstack.createQuery(
			() => helpers.queryHookArgs(client, path, input(), opts?.()) as any,
		);
	}

	function createMutation(
		path: string[],
		...[opts]: [
			| solid.Accessor<
					queryCore.WrapMutationOptions<tanstack.SolidMutationOptions>
			  >
			| undefined,
		]
	) {
		const client = useClient();
		return tanstack.createMutation(() =>
			helpers.mutationHookArgs(client, path, opts?.()),
		);
	}

	function createSubscription(
		path: string[],
		...[input, opts]: [
			solid.Accessor<unknown | tanstack.SkipToken>,
			opts:
				| solid.Accessor<queryCore.SubscriptionOptions<unknown, unknown>>
				| undefined,
		]
	) {
		const enabled = () => opts?.().enabled ?? true;
		const client = useClient();

		solid.createEffect(
			solid.on(
				() => [input(), enabled()],
				([input]) => {
					const unsubscribe = helpers.handleSubscription(
						client,
						path,
						input,
						opts,
					);

					solid.onCleanup(() => unsubscribe?.());
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
		useUtils: () => queryCore.createQueryUtilsProxy(useContext()),
	} satisfies SolidQueryProxyBuiltins<P>;

	return Object.assign(hooks, {
		createQuery,
		createMutation,
		createSubscription,
	});
}
