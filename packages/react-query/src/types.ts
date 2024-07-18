import type * as rspc from "@rspc/client";
import type * as queryCore from "@rspc/query-core";
import type * as tanstack from "@tanstack/react-query";
import type * as react from "react";

export interface ProviderProps<P extends rspc.Procedures>
	extends react.PropsWithChildren {
	client: rspc.Client<P>;
	queryClient: tanstack.QueryClient;
}

export type ReactQueryProxyBuiltins<P extends rspc.Procedures> = {
	Provider: react.FunctionComponent<ProviderProps<P>>;
	useUtils(): queryCore.UtilsProceduresProxy<P>;
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
> = P["kind"] extends "query"
	? {
			useQuery(
				input: rspc.VoidIfInputNull<P, P["input"] | tanstack.SkipToken>,
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
				input: rspc.VoidIfInputNull<P, P["input"] | tanstack.SkipToken>,
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
	: P["kind"] extends "mutation"
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
		: P["kind"] extends "subscription"
			? {
					useSubscription(
						input: rspc.VoidIfInputNull<P, P["input"] | tanstack.SkipToken>,
						opts?: Partial<
							rspc.SubscriptionObserver<rspc.ProcedureResult<P>, unknown>
						>,
					): void;
				}
			: never;
