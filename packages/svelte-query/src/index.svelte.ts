import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/svelte-query";
import { getRspcClientContext } from "./context";
import { onDestroy } from "svelte";

export * from "@rspc/query-core";

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
		keyAndInput: () => queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: () => queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.UndefinedInitialDataOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.CreateQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: () => queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: () => queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.DefinedInitialDataOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.DefinedCreateQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: () => queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: () => queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.CreateQueryOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.CreateBaseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	> {
		return tanstack.createQuery(
			() => helpers.useQueryArgs(keyAndInput(), opts?.()) as any,
		);
	}

	function createMutation<
		K extends rspc.inferMutations<TProcedures>["key"] & string,
		TContext = unknown,
	>(
		key: () => K,
		opts?: () => queryCore.WrapMutationOptions<
			TProcedures,
			tanstack.CreateMutationOptions<
				rspc.inferMutationResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferMutationInput<TProcedures, K> extends never
					? undefined
					: rspc.inferMutationInput<TProcedures, K>,
				TContext
			>
		>,
	) {
		return tanstack.createMutation(
			() => helpers.useMutationArgs(key(), opts?.()) as any,
		);
	}

	function createSubscription<
		K extends rspc.inferSubscriptions<TProcedures>["key"] & string,
	>(
		keyAndInput: () => queryCore.KeyAndInputSkipToken<
			TProcedures,
			K,
			"subscriptions"
		>,
		opts: () => queryCore.SubscriptionOptions<TProcedures, K>,
	) {
		$effect(() => {
			const unsubscribe = helpers.handleSubscription(
				keyAndInput(),
				opts,
				helpers.useClient(),
			);

			return () => unsubscribe?.();
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
