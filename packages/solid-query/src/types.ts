import type * as rspc from "@rspc/client";
import type * as queryCore from "@rspc/query-core";
import type * as tanstack from "@tanstack/solid-query";
import type * as solid from "solid-js";

export interface ProviderProps<P extends rspc.Procedures>
	extends solid.ParentProps {
	client: rspc.Client<P>;
	queryClient: tanstack.QueryClient;
}

export type SolidQueryProxyBuiltins<P extends rspc.Procedures> = {
	Provider: solid.Component<ProviderProps<P>>;
	useUtils(): queryCore.UtilsProceduresProxy<P>;
};

export type SolidQueryProceduresProxy<
	P extends rspc.Procedures,
	TPath extends string = "",
> = {
	[K in keyof P]: K extends string
		? P[K] extends rspc.Procedure
			? ProcedureProxyMethods<P[K], rspc.JoinPath<TPath, K>>
			: P[K] extends rspc.Procedures
				? SolidQueryProceduresProxy<P[K], rspc.JoinPath<TPath, K>>
				: never
		: never;
};

export type SolidQueryProxy<P extends rspc.Procedures> = Omit<
	SolidQueryProceduresProxy<P>,
	keyof SolidQueryProxyBuiltins<P>
> &
	SolidQueryProxyBuiltins<P>;

export type ProcedureProxyMethods<
	P extends rspc.Procedure,
	TPath extends string,
> = P["variant"] extends "query"
	? {
			createQuery(
				input: solid.Accessor<P["input"] | tanstack.SkipToken>,
				opts?: solid.Accessor<
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
				input: solid.Accessor<P["input"] | tanstack.SkipToken>,
				opts?: solid.Accessor<
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
					opts?: solid.Accessor<
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
						input: solid.Accessor<P["input"] | tanstack.SkipToken>,
						opts?: solid.Accessor<
							Partial<
								rspc.SubscriptionObserver<rspc.ProcedureResult<P>, unknown>
							>
						>,
					): void;
				}
			: never;
