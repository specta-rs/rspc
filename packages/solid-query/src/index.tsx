import type { VoidIfInputNull } from "@rspc/client/next";
import {
	type Client,
	type Procedure,
	type Procedures,
	createProceduresProxy,
	traverseClient,
} from "@rspc/client/next";
import * as tanstack from "@tanstack/solid-query";

// TODO: these should be overloaded to support passing options object when input is null
export type RspcQueryOptions<P extends Procedure> = {
	<TQueryFnData extends P["output"], TData = TQueryFnData>(
		input: VoidIfInputNull<P> | tanstack.SkipToken,
		options?: Omit<
			ReturnType<
				tanstack.UndefinedInitialDataOptions<
					TQueryFnData,
					P["error"],
					TData,
					any
				>
			>,
			"queryKey" | "queryFn" | "queryHash" | "queryHashFn"
		>,
	): tanstack.UndefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>;
	<TQueryFnData extends P["output"], TData = TQueryFnData>(
		input: VoidIfInputNull<P> | tanstack.SkipToken,
		options?: Omit<
			ReturnType<
				tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>
			>,
			"queryKey" | "queryFn" | "queryHash" | "queryHashFn"
		>,
	): tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>;
};

export type QueryMethods<P extends Procedure> = {
	queryOptions: RspcQueryOptions<P>;

	queryKey: (
		input?: Partial<VoidIfInputNull<P>>,
	) => tanstack.DataTag<ReadonlyArray<unknown>, P["output"], P["error"]>;
};

type RspcMutationOptions<P extends Procedure> = <TCtx = unknown>(
	options?: Omit<
		ReturnType<
			tanstack.UseMutationOptions<P["output"], P["error"], P["input"], TCtx>
		>,
		"mutationKey" | "mutationFn"
	>,
) => tanstack.UseMutationOptions<P["output"], P["error"], P["input"], TCtx>;

export type MutationMethods<P extends Procedure> = {
	mutationOptions: RspcMutationOptions<P>;

	mutationKey: () => ReadonlyArray<unknown>;
};

export type ProcedureProxyMethods<P extends Procedure> =
	P["kind"] extends "query"
		? QueryMethods<P>
		: P["kind"] extends "mutation"
			? MutationMethods<P>
			: // : P["kind"] extends "subscription"
				// ? { subscribe: SubscriptionResolver<P> }
				never;

export type OptionsProceduresProxy<P extends Procedures> = {
	[K in keyof P]: P[K] extends Procedure
		? ProcedureProxyMethods<P[K]>
		: P[K] extends Procedures
			? OptionsProceduresProxy<P[K]>
			: never;
};

type UtilsMethods = keyof QueryMethods<any> | keyof MutationMethods<any>;

export function createRSPCOptionsProxy<P extends Procedures>(
	client: Client<P>,
): OptionsProceduresProxy<P> {
	return createProceduresProxy<OptionsProceduresProxy<P>>(({ args, path }) => {
		const option = path.pop() as UtilsMethods;

		const methods: Record<UtilsMethods, () => unknown> = {
			queryOptions: () => {
				return () =>
					tanstack.queryOptions({
						...(args[1] ?? {}),
						queryKey: [path, args[0]],
						queryFn:
							args[0] === tanstack.skipToken
								? args[0]
								: () => (traverseClient(client, path) as any).query(args[0]),
					});
			},
			queryKey: () => {
				return [path, args[0]];
			},
			mutationOptions: () => {
				return () => ({
					...(args[0] ?? {}),
					mutationKey: [path],
					mutationFn: (input: unknown) =>
						(traverseClient(client, path) as any).mutate(input),
				});
			},
			mutationKey: () => {
				return [path];
			},
		};

		if (option in methods) {
			return methods[option]();
		}

		throw new Error();
	});
}
