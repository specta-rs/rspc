import type * as rspc from "@rspc/client";
import type * as queryCore from "@rspc/query-core";
import type * as tanstack from "@tanstack/react-query";
import type * as react from "react";

export interface Context<P extends rspc.Procedures> {
	client: rspc.Client<P>;
	queryClient: tanstack.QueryClient;
}

export interface ProviderProps<P extends rspc.Procedures>
	extends react.PropsWithChildren {
	client: rspc.Client<P>;
	queryClient: tanstack.QueryClient;
}

export type ReactQueryProxyBuiltins<P extends rspc.Procedures> = {
	Provider: react.FunctionComponent<ProviderProps<P>>;
};

export type ReactQueryProceduresProxy<
	P extends rspc.Procedures,
	TPath extends string = "",
> = {
	[K in keyof P]: K extends string
		? P[K] extends rspc.Procedure
			? ProcedureProxyMethods<P[K], rspc.JoinPath<TPath, K>>
			: P[K] extends rspc.Procedures
				? ReactQueryProceduresProxy<P[K], rspc.JoinPath<TPath, K>>
				: never
		: never;
};

export type ReactQueryProxy<P extends rspc.Procedures> = Omit<
	ReactQueryProceduresProxy<P>,
	keyof ReactQueryProxyBuiltins<P>
> &
	ReactQueryProxyBuiltins<P>;

export type ProcedureProxyMethods<
	P extends rspc.Procedure,
	TPath extends string,
> = P["variant"] extends "query"
	? {
			useQuery(
				input: P["input"] | tanstack.SkipToken,
				opts?: queryCore.WrapQueryOptions<
					tanstack.DefinedInitialDataOptions<
						rspc.ProcedureResult<P>,
						unknown,
						rspc.ProcedureResult<P>,
						[TPath, P["input"]]
					>
				>,
			): tanstack.DefinedUseQueryResult<rspc.ProcedureResult<P>, unknown>;
			useQuery(
				input: P["input"] | tanstack.SkipToken,
				opts?: queryCore.WrapQueryOptions<
					tanstack.UndefinedInitialDataOptions<
						rspc.ProcedureResult<P>,
						unknown,
						rspc.ProcedureResult<P>,
						[TPath, P["input"]]
					>
				>,
			): tanstack.UseQueryResult<rspc.ProcedureResult<P>, unknown>;
		}
	: P["variant"] extends "mutation"
		? {
				useMutation<TContext = unknown>(
					opts?: tanstack.UseMutationOptions<
						rspc.ProcedureResult<P>,
						unknown,
						P["input"],
						TContext
					>,
				): tanstack.UseMutationResult<
					rspc.ProcedureResult<P>,
					unknown,
					P["input"],
					TContext
				>;
			}
		: P["variant"] extends "subscription"
			? {
					useSubscription(
						input: P["input"],
						opts?: Partial<
							rspc.SubscriptionObserver<rspc.ProcedureResult<P>, unknown>
						>,
					): void;
				}
			: never;
