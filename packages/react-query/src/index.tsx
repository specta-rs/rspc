import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/react-query";
import * as react from "react";

export * from "@rspc/query-core";

export function createReactQueryHooks<
	TProceduresLike extends rspc.ProceduresDef,
>() {
	type TProcedures = rspc.inferProcedures<TProceduresLike>;

	const Context = react.createContext<queryCore.Context<TProcedures> | null>(
		null,
	);

	const helpers = queryCore.createQueryHookHelpers({
		useContext: () => react.useContext(Context),
	});

	function useContext() {
		const ctx = react.useContext(Context);
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

	function useQuery<K extends rspc.inferQueries<TProcedures>["key"] & string>(
		keyAndInput: queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.DefinedInitialDataOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.DefinedUseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function useQuery<K extends rspc.inferQueries<TProcedures>["key"] & string>(
		keyAndInput: queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.UndefinedInitialDataOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.UseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function useQuery<K extends rspc.inferQueries<TProcedures>["key"] & string>(
		keyAndInput: queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.UseQueryOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.UseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function useQuery<K extends rspc.inferQueries<TProcedures>["key"] & string>(
		keyAndInput: queryCore.KeyAndInputSkipToken<TProcedures, K>,
		opts?: queryCore.WrapQueryOptions<
			TProcedures,
			tanstack.UseQueryOptions<
				rspc.inferQueryResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<TProcedures, K>,
				queryCore.KeyAndInputSkipToken<TProcedures, K>
			>
		>,
	): tanstack.QueryObserverResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	> {
		return tanstack.useQuery(helpers.useQueryArgs(keyAndInput, opts) as any);
	}

	function useMutation<
		K extends rspc.inferMutations<TProcedures>["key"] & string,
		TContext = unknown,
	>(
		key: K,
		opts?: queryCore.WrapMutationOptions<
			TProcedures,
			tanstack.UseMutationOptions<
				rspc.inferMutationResult<TProcedures, K>,
				rspc.RSPCError,
				rspc.inferMutationInput<TProcedures, K> extends never
					? undefined
					: rspc.inferMutationInput<TProcedures, K>,
				TContext
			>
		>,
	) {
		return tanstack.useMutation(helpers.useMutationArgs(key, opts));
	}

	function useSubscription<
		K extends TProcedures["subscriptions"]["key"] & string,
	>(
		keyAndInput: queryCore.KeyAndInputSkipToken<
			TProcedures,
			K,
			"subscriptions"
		>,
		opts: queryCore.SubscriptionOptions<TProcedures, K>,
	) {
		const queryKey = tanstack.hashKey(keyAndInput);

		const enabled = opts?.enabled ?? true;

		const optsRef = react.useRef<typeof opts>(opts);
		optsRef.current = opts;

		// biome-ignore lint/correctness/useExhaustiveDependencies:
		return react.useEffect(
			() =>
				helpers.handleSubscription(
					keyAndInput,
					() => optsRef.current,
					helpers.useClient(),
				),
			[queryKey, enabled],
		);
	}

	return {
		_rspc_def: undefined! as TProcedures, // This allows inferring the operations type from TS helpers
		Provider: ({
			children,
			client,
			queryClient,
		}: {
			children?: react.ReactElement;
			client: rspc.Client<TProcedures>;
			queryClient: tanstack.QueryClient;
		}) => (
			<Context.Provider
				value={{
					client,
					queryClient,
				}}
			>
				<tanstack.QueryClientProvider client={queryClient}>
					{children}
				</tanstack.QueryClientProvider>
			</Context.Provider>
		),
		useContext,
		useUtils,
		useQuery,
		useMutation,
		useSubscription,
	};
}
