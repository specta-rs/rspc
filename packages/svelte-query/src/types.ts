import type * as rspc from "@rspc/client";
import type * as queryCore from "@rspc/query-core";
import type * as tanstack from "@tanstack/svelte-query";

export type SvelteQueryProxyBuiltins<P extends rspc.Procedures> = {
	useUtils(): queryCore.UtilsProceduresProxy<P>;
};

export type SvelteQueryProceduresProxy<
	P extends rspc.Procedures,
	TPath extends string = "",
> = {
	[K in keyof P]: K extends string
		? P[K] extends rspc.Procedure
			? ProcedureProxyMethods<P[K], rspc.JoinPath<TPath, K>>
			: P[K] extends rspc.Procedures
				? SvelteQueryProceduresProxy<P[K], rspc.JoinPath<TPath, K>>
				: never
		: never;
};

export type SvelteQueryProxy<P extends rspc.Procedures> = Omit<
	SvelteQueryProceduresProxy<P>,
	keyof SvelteQueryProxyBuiltins<P>
> &
	SvelteQueryProxyBuiltins<P>;

export type ProcedureProxyMethods<
	P extends rspc.Procedure,
	TPath extends string,
> = P["variant"] extends "query"
	? {
			createQuery(
				input: tanstack.StoreOrVal<P["input"] | tanstack.SkipToken>,
				opts?: tanstack.StoreOrVal<
					queryCore.WrapQueryOptions<
						tanstack.DefinedInitialDataOptions<
							rspc.ProcedureResult<P>,
							unknown,
							rspc.ProcedureResult<P>,
							[TPath, P["input"]]
						>
					>
				>,
			): tanstack.DefinedCreateQueryResult<
				rspc.Result<P["result"], P["error"]>,
				unknown
			>;
			createQuery(
				input: tanstack.StoreOrVal<P["input"] | tanstack.SkipToken>,
				opts?: tanstack.StoreOrVal<
					queryCore.WrapQueryOptions<
						tanstack.UndefinedInitialDataOptions<
							rspc.ProcedureResult<P>,
							unknown,
							rspc.ProcedureResult<P>,
							[TPath, P["input"]]
						>
					>
				>,
			): tanstack.CreateQueryResult<
				rspc.Result<P["result"], P["error"]>,
				unknown
			>;
		}
	: P["variant"] extends "mutation"
		? {
				createMutation<TContext = unknown>(
					opts?: tanstack.StoreOrVal<
						tanstack.CreateMutationOptions<
							rspc.ProcedureResult<P>,
							unknown,
							P["input"],
							TContext
						>
					>,
				): tanstack.CreateMutationResult<
					rspc.ProcedureResult<P>,
					unknown,
					P["input"],
					TContext
				>;
			}
		: P["variant"] extends "subscription"
			? {
					createSubscription(
						input: tanstack.StoreOrVal<P["input"]>,
						opts?: tanstack.StoreOrVal<
							Partial<
								rspc.SubscriptionObserver<rspc.ProcedureResult<P>, unknown>
							>
						>,
					): void;
				}
			: never;
