import type * as rspc from "@rspc/client";
import * as tanstack from "@tanstack/query-core";

export function createTanstackQueryProxy<
	TProceduresLike extends rspc.ProceduresDef,
	TTanstack extends {
		queryOptions: (options: any) => any;
		infiniteQueryOptions: (options: any) => any;
	},
>() {
	type TProcedures = rspc.inferProcedures<TProceduresLike>;

	return function (args: {
		queryOptions: TTanstack["queryOptions"];
		infiniteQueryOptions: TTanstack["infiniteQueryOptions"];
	}) {
		return {
			// queryOptions: args.queryOptions,
			// mutationOptions: ():  => {},
		};
	};
}

export interface BaseOptions<TProcedures extends rspc.ProceduresDef> {
	rspc?: { client?: rspc.Client<TProcedures> };
}

export interface SubscriptionOptions<
	P extends rspc.ProceduresDef,
	K extends rspc.inferSubscriptions<P>["key"] & string,
> extends rspc.SubscriptionOptions<rspc.inferSubscriptionResult<P, K>> {
	enabled?: boolean;
	client?: rspc.Client<P>;
}

export function createRSPCQueryUtils<
	TProceduresLike extends rspc.ProceduresDef,
>(
	client: rspc.Client<rspc.inferProcedures<TProceduresLike>>,
	queryClient: tanstack.QueryClient,
) {
	type TProcedures = rspc.inferProcedures<TProceduresLike>;
	type TBaseOptions = BaseOptions<TProcedures>;
	type QueryKey<K extends string, TProcedures extends rspc.ProceduresLike> = [
		key: K,
		...input: rspc._inferProcedureHandlerInput<TProcedures, "queries", K>,
	];
	type AllowedKeys = TProcedures["queries"]["key"] & string;
	type RSPCOptions<T> = Omit<T, "queryKey" | "queryFn"> & TBaseOptions;
	type RSPCFetchQueryOptions<
		K extends string,
		TData = rspc.inferQueryResult<TProcedures, K>,
	> = RSPCOptions<
		tanstack.FetchQueryOptions<
			TData,
			rspc.RSPCError,
			TData,
			QueryKey<K, TProcedures>
		>
	>;

	type RSPCEnsureFetchQueryOptions<
		K extends string,
		TData = rspc.inferQueryResult<TProcedures, K>,
	> = RSPCOptions<
		tanstack.EnsureQueryDataOptions<
			TData,
			rspc.RSPCError,
			TData,
			QueryKey<K, TProcedures>
		>
	>;

	return {
		fetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCFetchQueryOptions<K>,
		) =>
			queryClient.fetchQuery({
				...opts,
				queryKey,
				queryFn: () => client.query(queryKey),
			}),
		prefetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCFetchQueryOptions<K>,
		) =>
			queryClient.prefetchQuery({
				...opts,
				queryKey,
				queryFn: () => client.query(queryKey),
			}),
		ensureData: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCEnsureFetchQueryOptions<K>,
		) =>
			queryClient.ensureQueryData({
				...opts,
				queryKey,
				queryFn: () => client.query(queryKey),
			}),
		invalidate: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<tanstack.InvalidateQueryFilters, "queryKey">,
			opts?: tanstack.InvalidateOptions,
		) => queryClient.invalidateQueries({ ...filters, queryKey }, opts),
		refetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<tanstack.RefetchQueryFilters, "queryKey">,
			opts?: tanstack.RefetchOptions,
		) => queryClient.refetchQueries({ ...filters, queryKey }, opts),
		cancel: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<tanstack.QueryFilters, "queryKey">,
			opts?: tanstack.CancelOptions,
		) => queryClient.cancelQueries({ ...filters, queryKey }, opts),
		setData: <
			K extends AllowedKeys,
			TData = rspc.inferQueryResult<TProcedures, K>,
		>(
			queryKey: QueryKey<K, TProcedures>,
			updater: tanstack.Updater<TData | undefined, TData | undefined>,
			options?: tanstack.SetDataOptions,
		) => {
			queryClient.setQueryData<TData>(queryKey, updater, options);
		},
		getData: <
			K extends AllowedKeys,
			TData = rspc.inferQueryResult<TProcedures, K>,
		>(
			queryKey: QueryKey<K, TProcedures>,
		) => queryClient.getQueryData<TData>(queryKey),
	};
}

export interface Context<TProcedures extends rspc.ProceduresDef> {
	client: rspc.Client<TProcedures>;
	queryClient: tanstack.QueryClient;
}

export type KeyAndInputSkipToken<
	P extends rspc.ProceduresDef,
	K extends rspc.inferQueries<P>["key"] & string,
	O extends keyof rspc.ProceduresDef = "queries",
> = [
	key: K,
	...input: rspc._inferProcedureHandlerInput<P, O, K> | [tanstack.SkipToken],
];

export type HookOptions<P extends rspc.ProceduresDef, T> = T & BaseOptions<P>;
export type WrapQueryOptions<P extends rspc.ProceduresDef, T> = Omit<
	T,
	"queryKey" | "queryFn"
> &
	BaseOptions<P>;
export type WrapMutationOptions<P extends rspc.ProceduresDef, T> = Omit<
	T,
	"_defaulted" | "variables" | "mutationKey"
> &
	BaseOptions<P>;

function isSkipTokenInput(array: any[]): array is [tanstack.SkipToken] {
	return array.length === 1 && array[0] === tanstack.skipToken;
}

export function createQueryHookHelpers<P extends rspc.ProceduresDef>(args: {
	useContext(): Context<P> | null;
}) {
	type TBaseOptions = BaseOptions<P>;

	function useContext() {
		const ctx = args.useContext();
		if (!ctx) throw new Error("rspc context provider not found!");
		return ctx;
	}

	function useClient() {
		return useContext().client;
	}

	function getClient<T extends TBaseOptions>(opts?: T) {
		return opts?.rspc?.client ?? useClient();
	}

	function useQueryArgs<
		K extends rspc.inferQueries<P>["key"] & string,
		O extends WrapQueryOptions<
			P,
			tanstack.QueryObserverOptions<
				rspc.inferQueryResult<P, K>,
				rspc.RSPCError,
				rspc.inferQueryResult<P, K>,
				rspc.inferQueryResult<P, K>,
				KeyAndInputSkipToken<P, K>
			>
		>,
	>(keyAndInput: KeyAndInputSkipToken<P, K>, opts?: O) {
		const client = getClient(opts);
		const [key, ...input] = keyAndInput;

		return {
			...opts,
			queryKey: keyAndInput,
			queryFn: isSkipTokenInput(input)
				? (input[0] as tanstack.SkipToken)
				: () => client.query([key, ...input]),
		};
	}

	type MutationObserverOptions<
		K extends rspc.inferMutations<P>["key"] & string,
	> = WrapMutationOptions<
		P,
		tanstack.MutationObserverOptions<
			rspc.inferMutationResult<P, K>,
			rspc.RSPCError,
			rspc.inferMutationInput<P, K> extends never
				? undefined
				: rspc.inferMutationInput<P, K>,
			any
		>
	>;

	function useMutationArgs<
		K extends rspc.inferMutations<P>["key"] & string,
		O extends MutationObserverOptions<K>,
	>(key: K, opts?: O) {
		const client = getClient(opts);

		return {
			...opts,
			mutationKey: [key],
			mutationFn: (input) =>
				client.mutation([key, ...(input ? [input] : [])] as any),
		} satisfies tanstack.MutationObserverOptions<
			rspc.inferMutationResult<P, K>,
			rspc.RSPCError,
			rspc.inferMutationInput<P, K> extends never
				? undefined
				: rspc.inferMutationInput<P, K>,
			any
		>;
	}

	function handleSubscription<
		K extends rspc.inferSubscriptions<P>["key"] & string,
	>(
		keyAndInput: KeyAndInputSkipToken<P, K, "subscriptions">,
		_opts: () => SubscriptionOptions<P, K>,
		_client: rspc.Client<P>,
	) {
		// function to allow options to be passed in via ref
		const opts = _opts();
		const [key, ...input] = keyAndInput;

		if (!(opts.enabled ?? true) || isSkipTokenInput(input)) return;

		const client = opts.client ?? _client;
		let isStopped = false;

		const unsubscribe = client.addSubscription([key, ...input], {
			onStarted: () => {
				if (!isStopped) opts.onStarted?.();
			},
			onData: (d) => {
				if (!isStopped) opts.onData(d);
			},
			onError: (e) => {
				if (!isStopped) opts.onError?.(e);
			},
		});

		return () => {
			isStopped = true;
			unsubscribe();
		};
	}

	return {
		useContext,
		getClient,
		useClient,
		useExtractOps: getClient,
		useQueryArgs,
		useMutationArgs,
		handleSubscription,
	};
}
