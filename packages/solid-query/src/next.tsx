import {
	Client,
	createProceduresProxy,
	Procedure,
	Procedures,
	traverseClient,
} from "@rspc/client/next";
import * as tanstack from "@tanstack/solid-query";

export type QueryMethods<P extends Procedure> = {
	queryOptions<TQueryFnData extends P["output"], TData = TQueryFnData>(
		input: P["input"] | tanstack.SkipToken,
		options: Omit<
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
	): ReturnType<
		tanstack.UndefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>
	> & {
		queryKey: tanstack.DataTag<any, TQueryFnData>;
	};
	queryOptions<TQueryFnData extends P["output"], TData = TQueryFnData>(
		input: P["input"] | tanstack.SkipToken,
		options: Omit<
			ReturnType<
				tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>
			>,
			"queryKey" | "queryFn" | "queryHash" | "queryHashFn"
		>,
	): ReturnType<
		tanstack.DefinedInitialDataOptions<TQueryFnData, P["error"], TData, any>
	> & {
		queryKey: tanstack.DataTag<any, TQueryFnData>;
	};
};

export type ProcedureProxyMethods<P extends Procedure> =
	P["kind"] extends "query" ? { query: QueryMethods<P> } : never;
// P["kind"] extends "mutation"
// 		? { mutate: Resolver<P> }
// 		: P["kind"] extends "subscription"
// 			? { subscribe: SubscriptionResolver<P> }
// 			: never;

export type OptionsProceduresProxy<P extends Procedures> = {
	[K in keyof P]: P[K] extends Procedure
		? ProcedureProxyMethods<P[K]>
		: P[K] extends Procedures
			? OptionsProceduresProxy<P[K]>
			: never;
};

export function createRSPCOptionsProxy<P extends Procedures>(
	client: Client<P>,
) {
	return createProceduresProxy<OptionsProceduresProxy<P>>(({ args, path }) => {
		const option = path.pop();

		if (option === "queryOptions") {
			return tanstack.queryOptions({
				...args[1],
				queryKey: [path, args[0]],
				queryFn:
					args[0] === tanstack.skipToken
						? args[0]
						: () => (traverseClient(client, path) as any).query(args[0]),
			});
		}

		throw new Error();
	});
}
