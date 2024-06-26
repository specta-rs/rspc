import {
	type Client,
	type ProceduresDef,
	type ProceduresLike,
	type RSPCError,
	_inferInfiniteQueryProcedureHandlerInput,
	type _inferProcedureHandlerInput,
	type inferProcedures,
	type inferQueryResult,
} from "@rspc/client";
import type {
	CancelOptions,
	EnsureQueryDataOptions,
	FetchQueryOptions,
	InvalidateOptions,
	InvalidateQueryFilters,
	QueryClient,
	QueryFilters,
	RefetchOptions,
	RefetchQueryFilters,
	SetDataOptions,
	Updater,
} from "@tanstack/query-core";

export interface BaseOptions<TProcedures extends ProceduresDef> {
	rspc?: { client?: Client<TProcedures> };
}

export function createRSPCQueryUtils<TProceduresLike extends ProceduresDef>(
	client: Client<inferProcedures<TProceduresLike>>,
	queryClient: QueryClient,
) {
	type TProcedures = inferProcedures<TProceduresLike>;
	type TBaseOptions = BaseOptions<TProcedures>;
	type QueryKey<K extends string, TProcedures extends ProceduresLike> = [
		key: K,
		...input: _inferProcedureHandlerInput<TProcedures, "queries", K>,
	];
	type AllowedKeys = TProcedures["queries"]["key"] & string;
	type RSPCOptions<T> = Omit<T, "queryKey" | "queryFn"> & TBaseOptions;
	type RSPCFetchQueryOptions<
		K extends string,
		TData = inferQueryResult<TProcedures, K>,
	> = RSPCOptions<
		FetchQueryOptions<TData, RSPCError, TData, QueryKey<K, TProcedures>>
	>;

	type RSPCEnsureFetchQueryOptions<
		K extends string,
		TData = inferQueryResult<TProcedures, K>,
	> = RSPCOptions<
		EnsureQueryDataOptions<TData, RSPCError, TData, QueryKey<K, TProcedures>>
	>;

	return {
		fetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCFetchQueryOptions<K>,
		) =>
			queryClient.fetchQuery({
				...(opts as any),
				queryKey,
				queryFn: client.query(queryKey as any),
			}),
		prefetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCFetchQueryOptions<K>,
		) =>
			queryClient.prefetchQuery({
				...(opts as any),
				queryKey,
				queryFn: () => client.query(queryKey as any),
			}),
		ensureData: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			opts?: RSPCEnsureFetchQueryOptions<K>,
		) =>
			queryClient.ensureQueryData({
				...(opts as any),
				queryKey,
				queryFn: () => client.query(queryKey as any),
			}),
		invalidate: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<InvalidateQueryFilters, "queryKey">,
			opts?: InvalidateOptions,
		) => queryClient.invalidateQueries({ ...filters, queryKey }, opts),
		refetch: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<RefetchQueryFilters, "queryKey">,
			opts?: RefetchOptions,
		) => queryClient.refetchQueries({ ...filters, queryKey }, opts),
		cancel: <K extends AllowedKeys>(
			queryKey: QueryKey<K, TProcedures>,
			filters?: Omit<QueryFilters, "queryKey">,
			opts?: CancelOptions,
		) => queryClient.cancelQueries({ ...filters, queryKey }, opts),
		setData: <K extends AllowedKeys, TData = inferQueryResult<TProcedures, K>>(
			queryKey: QueryKey<K, TProcedures>,
			updater: Updater<TData | undefined, TData | undefined>,
			options?: SetDataOptions,
		) => {
			queryClient.setQueryData<TData>(queryKey, updater, options);
		},
		getData: <K extends AllowedKeys, TData = inferQueryResult<TProcedures, K>>(
			queryKey: QueryKey<K, TProcedures>,
		) => queryClient.getQueryData<TData>(queryKey),
	};
}
