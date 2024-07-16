import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/svelte-query";
import { derived, get, readable, type Readable } from "svelte/store";

import { getRspcClientContext } from "./context";
import type { SvelteQueryProxyBuiltins } from "./types";

export function isSvelteStore<T>(
	obj: tanstack.StoreOrVal<T>,
): obj is Readable<T> {
	return (
		typeof obj === "object" &&
		obj !== null &&
		"subscribe" in obj &&
		typeof obj.subscribe === "function"
	);
}

export function enforceSvelteStore<T>(
	obj: tanstack.StoreOrVal<T>,
): Readable<T> {
	if (isSvelteStore(obj)) return obj;

	return readable(obj);
}

export function createHooks<P extends rspc.Procedures>() {
	const helpers = queryCore.createQueryHooksHelpers<P>();

	function useContext() {
		const ctx = getRspcClientContext<P>();
		if (ctx?.queryClient === undefined)
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
			tanstack.StoreOrVal<unknown | tanstack.SkipToken>,
			(
				| tanstack.StoreOrVal<
						queryCore.WrapQueryOptions<tanstack.CreateQueryOptions>
				  >
				| undefined
			),
		]
	) {
		const client = useClient();
		return tanstack.createQuery(
			derived(
				[enforceSvelteStore(input), enforceSvelteStore(opts ?? {})],
				([$input, $opts]) =>
					helpers.queryHookArgs(client, path, $input, $opts) as any,
			),
		);
	}

	function createMutation(
		path: string[],
		...[opts]: [
			| tanstack.StoreOrVal<
					queryCore.WrapMutationOptions<tanstack.CreateMutationOptions>
			  >
			| undefined,
		]
	) {
		const client = useClient();
		return tanstack.createMutation(
			derived([enforceSvelteStore(opts ?? {})], ([$opts]) =>
				helpers.mutationHookArgs(client, path, $opts),
			),
		);
	}

	function createSubscription(
		path: string[],
		...[input, opts]: [
			tanstack.StoreOrVal<unknown>,
			opts:
				| tanstack.StoreOrVal<queryCore.SubscriptionOptions<unknown, unknown>>
				| undefined,
		]
	) {
		const client = useClient();
		const optsStore = enforceSvelteStore(opts ?? {});
		const enabled = derived([optsStore], ([opts]) => opts?.enabled ?? true);

		const state = derived(
			[enforceSvelteStore(input), enabled],
			([input, enabled]) => [input, enabled] as const,
		);

		let unsubscribe: (() => void) | undefined;
		state.subscribe(
			([input]) => {
				unsubscribe = helpers.handleSubscription(client, path, input, () =>
					get(optsStore),
				);
			},
			() => unsubscribe?.(),
		);
	}

	const hooks = {
		useUtils: () => queryCore.createQueryUtilsProxy(useContext()),
	} satisfies SvelteQueryProxyBuiltins<P>;

	return Object.assign(hooks, {
		createQuery,
		createMutation,
		createSubscription,
	});
}
