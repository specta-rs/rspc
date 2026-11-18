import type { JSX } from "@solidjs/web";
import type * as rspc from "@rspc/client";
import * as solid from "solid-js";

import * as queryCore from "@rspc/query-core";
import * as tanstack from "@tanstack/solid-query";

const proxy = queryCore.createTanstackQueryProxy<any, typeof tanstack>()({
	queryOptions: tanstack.queryOptions,
	infiniteQueryOptions: tanstack.infiniteQueryOptions,
});

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
				tanstack.QueryOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				> & {
					initialData?: undefined;
				}
			>
		>,
	): tanstack.UseQueryResult<
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
				tanstack.QueryOptions<
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
	): tanstack.DefinedUseQueryResult<
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
				tanstack.QueryOptions<
					rspc.inferQueryResult<TProcedures, K>,
					rspc.RSPCError,
					rspc.inferQueryResult<TProcedures, K>,
					queryCore.KeyAndInputSkipToken<TProcedures, K>
				>
			>
		>,
	): tanstack.UseBaseQueryResult<
		rspc.inferQueryResult<TProcedures, K>,
		rspc.RSPCError
	> {
		return tanstack.useQuery(() =>
			helpers.useQueryArgs(keyAndInput(), opts?.()),
		);
	}

	function createMutation<
		K extends rspc.inferMutations<TProcedures>["key"] & string,
		TContext = unknown,
	>(
		key: solid.Accessor<K>,
		opts?: solid.Accessor<
			queryCore.WrapMutationOptions<
				TProcedures,
				tanstack.MutationOptions<
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
		return tanstack.useMutation(() =>
			helpers.useMutationArgs(key(), opts?.()),
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
		solid.createEffect(
			() => [keyAndInput(), opts()] as const,
			([keyAndInput, opts]) => {
				const unsubscribe = helpers.handleSubscription(
					keyAndInput,
					() => opts,
					helpers.useClient(),
				);

				return () => unsubscribe?.();
			},
		);
	}

	return {
		_rspc_def: undefined! as TProceduresLike, // This allows inferring the operations type from TS helpers
		Provider: (props: {
			children?: JSX.Element;
			client: rspc.Client<TProcedures>;
			queryClient: tanstack.QueryClient;
		}): JSX.Element => {
			return (
				<Context
					value={{
						client: props.client,
						queryClient: props.queryClient,
					}}
				>
					<tanstack.QueryClientProvider client={props.queryClient}>
						{props.children}
					</tanstack.QueryClientProvider>
				</Context>
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
