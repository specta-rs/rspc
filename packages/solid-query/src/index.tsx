import type * as rspc from "@rspc/client";
import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/solid-query";
import * as solid from "solid-js";

export * from "@rspc/query-core";

export function createSolidQueryHooks<
	TProceduresLike extends rspc.ProceduresDef,
>() {
	type TProcedures = rspc.inferProcedures<TProceduresLike>;

	const Context = solid.createContext<queryCore.Context<TProcedures> | null>(
		null,
	);

	const helpers = queryCore.createQueryHookHelpers({
		useContext: () => solid.useContext(Context),
	});

	function useContext() {
		const ctx = solid.useContext(Context);
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
		keyAndInput: solid.Accessor<queryCore.KeyAndInputSkipToken<TProcedures, K>>,
		opts?: solid.Accessor<
			queryCore.WrapQueryOptions<
				TProcedures,
				tanstack.SolidQueryOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				> & {
					initialData?: undefined;
				}
			>
		>,
	): tanstack.CreateQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: solid.Accessor<queryCore.KeyAndInputSkipToken<TProcedures, K>>,
		opts?: solid.Accessor<
			queryCore.WrapQueryOptions<
				TProcedures,
				tanstack.SolidQueryOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				> & {
					initialData:
						| rspc.inferQueryResult<TProcedures, K>
						| (() => rspc.inferQueryResult<TProcedures, K>);
				}
			>
		>,
	): tanstack.DefinedCreateBaseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	>;
	function createQuery<
		K extends rspc.inferQueries<TProcedures>["key"] & string,
	>(
		keyAndInput: solid.Accessor<queryCore.KeyAndInputSkipToken<TProcedures, K>>,
		opts?: solid.Accessor<
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
	): tanstack.QueryObserverResult<
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
		key: solid.Accessor<K>,
		opts?: tanstack.FunctionedParams<
			queryCore.WrapMutationOptions<
				TProcedures,
				tanstack.SolidMutationOptions<
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
		return tanstack.createMutation(() =>
			helpers.useMutationArgs(key(), opts?.()),
		);
	}

	function createSubscription<
		K extends rspc.inferSubscriptions<TProcedures>["key"] & string,
	>(
		keyAndInput: () => [
			key: K,
			...input:
				| rspc._inferProcedureHandlerInput<TProcedures, "subscriptions", K>
				| [tanstack.SkipToken],
		],
		opts: () => queryCore.SubscriptionOptions<TProcedures, K>,
	) {
		solid.createEffect(
			solid.on(
				() => [keyAndInput(), opts()] as const,
				([keyAndInput, opts]) => {
					const unsubscribe = helpers.handleSubscription(
						keyAndInput,
						() => opts,
						helpers.useClient(),
					);

					solid.onCleanup(() => unsubscribe?.());
				},
			),
		);
	}

	return {
		_rspc_def: undefined! as TProceduresLike, // This allows inferring the operations type from TS helpers
		Provider: (props: {
			children?: solid.JSX.Element;
			client: rspc.Client<TProcedures>;
			queryClient: tanstack.QueryClient;
		}): solid.JSX.Element => {
			return (
				<Context.Provider
					value={{
						client: props.client,
						queryClient: props.queryClient,
					}}
				>
					<tanstack.QueryClientProvider client={props.queryClient}>
						{props.children}
					</tanstack.QueryClientProvider>
				</Context.Provider>
			);
		},
		useContext,
		useUtils,
		createQuery,
		// createInfiniteQuery,
		createMutation,
		createSubscription,
	};
}
