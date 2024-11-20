import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/svelte-query";
import { getRspcClientContext } from "./context";
import { derived, get, readable, type Readable } from "svelte/store";
import { onDestroy } from "svelte";

export * from "@rspc/query-core";

function isSvelteStore<T extends object>(
	obj: tanstack.StoreOrVal<T>,
): obj is Readable<T> {
	return "subscribe" in obj && typeof obj.subscribe === "function";
}

function enforceSvelteStore<T extends object>(
	obj: tanstack.StoreOrVal<T>,
): Readable<T> {
	if (isSvelteStore(obj)) {
		return obj;
	}
	return readable(obj);
}

export function createSvelteQueryHooks<
	TProceduresLike extends rspc.ProceduresDef,
>() {
	type TProcedures = rspc.inferProcedures<TProceduresLike>;

	const helpers = queryCore.createQueryHookHelpers({
		useContext: () => getRspcClientContext<TProcedures>(),
	});

	function useContext() {
		const ctx = getRspcClientContext<TProcedures>();
		if (ctx?.queryClient === undefined)
			throw new Error(
				"The rspc context has not been set. Ensure you have the <rspc.Provider> component higher up in your component tree.",
			);
		return ctx;
	}

	function useUtils() {
		const ctx = useContext();
		return queryCore.createRSPCQueryUtils(ctx.client, ctx.queryClient);
	}

	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: tanstack.StoreOrVal<
			queryCore.KeyAndInputSkipToken<TProcedures, K>
		>,
		opts?: tanstack.StoreOrVal<
			queryCore.WrapQueryOptions<
				TProcedures,
				tanstack.UndefinedInitialDataOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				>
			>
		>,
	): tanstack.CreateQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: tanstack.StoreOrVal<
			queryCore.KeyAndInputSkipToken<TProcedures, K>
		>,
		opts?: tanstack.StoreOrVal<
			queryCore.WrapQueryOptions<
				TProcedures,
				tanstack.DefinedInitialDataOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				>
			>
		>,
	): tanstack.DefinedCreateQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: tanstack.StoreOrVal<
			queryCore.KeyAndInputSkipToken<TProcedures, K>
		>,
		opts?: tanstack.StoreOrVal<
			queryCore.WrapQueryOptions<
				TProcedures,
				tanstack.CreateQueryOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				>
			>
		>,
	): tanstack.CreateBaseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	> {
		return tanstack.createQuery(
			derived(
				[enforceSvelteStore(keyAndInput), enforceSvelteStore(opts ?? {})],
				([$keyAndInput, $opts]) =>
					helpers.useQueryArgs($keyAndInput, $opts) as any,
			),
		);
	}

	function createMutation<
		K extends rspc.inferMutations<TProcedures>["key"] & string,
		TContext = unknown,
	>(
		key: K,
		opts?: tanstack.StoreOrVal<
			queryCore.WrapMutationOptions<
				TProcedures,
				tanstack.CreateMutationOptions<
					rspc.inferMutationResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferMutationInput<TProcedures, K> extends never
						? undefined
						: rspc.inferMutationInput<TProcedures, K>,
					TContext
				>
			>
		>,
	) {
		return tanstack.createMutation(
			derived([enforceSvelteStore(opts ?? {})], ([$opts]) =>
				helpers.useMutationArgs(key, $opts),
			),
		);
	}

	function createSubscription<
		K extends rspc.inferSubscriptions<TProcedures>["key"] & string,
	>(
		keyAndInput: tanstack.StoreOrVal<
			[
				key: K,
				...input:
					| rspc._inferProcedureHandlerInput<TProcedures, "subscriptions", K>
					| [tanstack.SkipToken],
			]
		>,
		opts: tanstack.StoreOrVal<queryCore.SubscriptionOptions<TProcedures, K>>,
	) {
		const optsStore = enforceSvelteStore(opts);

		enforceSvelteStore(keyAndInput).subscribe((keyAndInput) => {
			const unsubscribe = helpers.handleSubscription(
				keyAndInput,
				() => get(optsStore),
				helpers.useClient(),
			);

			onDestroy(() => unsubscribe?.());
		});
	}

	return {
		_rspc_def: undefined! as TProceduresLike,
		useContext,
		useUtils,
		createQuery,
		createMutation,
		createSubscription,
	};
}
