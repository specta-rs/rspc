import * as rspc from "@rspc/client";

import type * as tanstack from "@tanstack/query-core";
import type { Context, WrapQueryOptions } from ".";

export type FetchQueryOptions<
	P extends rspc.Procedure,
	TPath extends string,
> = WrapQueryOptions<
	tanstack.FetchQueryOptions<
		rspc.ProcedureResult<P>,
		unknown,
		rspc.ProcedureResult<P>,
		[TPath] | [TPath, P["input"]]
	>
>;

export type EnsureQueryDataOptions<
	P extends rspc.Procedure,
	TPath extends string,
> = WrapQueryOptions<
	tanstack.EnsureQueryDataOptions<
		rspc.ProcedureResult<P>,
		unknown,
		rspc.ProcedureResult<P>,
		[TPath] | [TPath, P["input"]]
	>
>;

export type QueryUtilsProxyMethods<
	P extends rspc.Procedure,
	TPath extends string,
> = {
	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientfetchquery
	 */
	fetch(
		input: P["input"],
		opts?: FetchQueryOptions<P, TPath>,
	): Promise<rspc.ProcedureResult<P>>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientprefetchquery
	 */
	prefetch(input: P["input"], opts?: FetchQueryOptions<P, TPath>): void;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientensurequerydata
	 */
	ensureData(
		input: P["input"],
		opts: EnsureQueryDataOptions<P, TPath>,
	): Promise<rspc.ProcedureResult<P>>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientinvalidatequeries
	 */
	invalidate(
		input?: P["input"],
		filters?: tanstack.InvalidateQueryFilters,
		options?: tanstack.InvalidateOptions,
	): Promise<void>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientrefetchqueries
	 */
	refetch(
		input?: P["input"],
		filters?: WrapQueryOptions<tanstack.RefetchQueryFilters>,
		options?: tanstack.RefetchOptions,
	): Promise<void>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientcancelqueries
	 */
	cancel(
		input?: P["input"],
		filters?: WrapQueryOptions<tanstack.QueryFilters>,
		options?: tanstack.CancelOptions,
	): Promise<void>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientresetqueries
	 */
	reset(input?: P["input"], options?: tanstack.ResetOptions): Promise<void>;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientsetquerydata
	 */
	setData(
		input: P["input"],
		updater: tanstack.Updater<
			rspc.ProcedureResult<P> | undefined,
			rspc.ProcedureResult<P> | undefined
		>,
		options?: tanstack.SetDataOptions,
	): void;

	/**
	 * @link https://tanstack.com/query/v5/docs/reference/QueryClient#queryclientgetquerydata
	 */
	getData(input?: P["input"]): rspc.ProcedureResult<P> | undefined;
};

export type UtilsProceduresProxy<
	P extends rspc.Procedures,
	TPath extends string = "",
> = {
	[K in keyof P]: K extends string
		? P[K] extends rspc.Procedure
			? P[K]["kind"] extends "query"
				? QueryUtilsProxyMethods<P[K], rspc.JoinPath<TPath, K>>
				: never
			: P[K] extends rspc.Procedures
				? UtilsProceduresProxy<P[K], rspc.JoinPath<TPath, K>>
				: never
		: never;
};

export function createQueryUtilsProxy<P extends rspc.Procedures>({
	client,
	queryClient,
}: Context<P>): UtilsProceduresProxy<P> {
	return rspc.createProceduresProxy(({ args, path }) => {
		const utils = {
			fetch() {
				const [input, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["fetch"]
				>;
				return queryClient.fetchQuery({
					...opts,
					queryKey: rspc.getQueryKey(path.join("."), input),
					queryFn: () =>
						rspc
							.traverseClient<rspc.ProcedureWithKind<"query">>(client, path)
							.query(input),
				});
			},
			prefetch() {
				const [input, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["prefetch"]
				>;
				return queryClient.prefetchQuery({
					...opts,
					queryKey: rspc.getQueryKey(path.join("."), input),
					queryFn: () =>
						rspc
							.traverseClient<rspc.ProcedureWithKind<"query">>(client, path)
							.query(input),
				});
			},
			ensureData() {
				const [input, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["ensureData"]
				>;
				return queryClient.ensureQueryData({
					...opts,
					queryKey: rspc.getQueryKey(path.join("."), input),
					queryFn: () =>
						rspc
							.traverseClient<rspc.ProcedureWithKind<"query">>(client, path)
							.query(input),
				});
			},
			invalidate() {
				const [input, filters, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["invalidate"]
				>;
				return queryClient.invalidateQueries(
					{ ...filters, queryKey: rspc.getQueryKey(path.join("."), input) },
					opts,
				);
			},
			refetch() {
				const [input, filters, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["refetch"]
				>;
				return queryClient.refetchQueries(
					{ ...filters, queryKey: rspc.getQueryKey(path.join("."), input) },
					opts,
				);
			},
			cancel() {
				const [input, filters, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["cancel"]
				>;
				return queryClient.cancelQueries(
					{ ...filters, queryKey: rspc.getQueryKey(path.join("."), input) },
					opts,
				);
			},
			setData() {
				const [input, updater, opts] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["setData"]
				>;
				return queryClient.setQueryData(
					rspc.getQueryKey(path.join("."), input),
					updater,
					opts,
				);
			},
			getData() {
				const [input] = args as Parameters<
					QueryUtilsProxyMethods<rspc.Procedure, string>["getData"]
				>;
				return queryClient.getQueryData(
					rspc.getQueryKey(path.join("."), input),
				);
			},
		};

		return utils[path.pop() as keyof typeof utils]();
	});
}
