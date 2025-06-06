import type { VoidIfInputNull, SubscriptionObserver } from "@rspc/client/next";
import { UntypedClient } from "@rspc/client/next";
import {
	type Client,
	type Procedure,
	type Procedures,
	createProceduresProxy,
	traverseClient,
} from "@rspc/client/next";
import * as tanstack from "@tanstack/solid-query";
import { type Accessor, createSignal } from "solid-js";

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

export type SubscriptionMethods<P extends Procedure> = {
	subscriptionOptions: (
		input: VoidIfInputNull<P>,
		opts: SubscriptionObserver<P["output"], P["error"]>,
	) => SubscriptionOptions<P["output"], P["error"]>;
};

export type ProcedureProxyMethods<P extends Procedure> =
	P["kind"] extends "query"
		? QueryMethods<P>
		: P["kind"] extends "mutation"
			? MutationMethods<P>
			: P["kind"] extends "subscription"
				? SubscriptionMethods<P>
				: never;

export type OptionsProceduresProxy<P extends Procedures> = {
	[K in keyof P]: P[K] extends Procedure
		? ProcedureProxyMethods<P[K]>
		: P[K] extends Procedures
			? OptionsProceduresProxy<P[K]>
			: never;
};

type UtilsMethods =
	| keyof QueryMethods<any>
	| keyof MutationMethods<any>
	| keyof SubscriptionMethods<any>;

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
			subscriptionOptions: () => {
				return {
					...args[1],
					subscribe: (innerOpts: SubscriptionObserver<any, any>) =>
						(traverseClient(client, path) as any).subscribe(args[0], innerOpts),
				};
			},
		};

		if (option in methods) {
			return methods[option]();
		}

		throw new Error(`Invalid proxy method '${option}'`);
	});
}

export interface SubscriptionOptions<TOut, TError>
	extends SubscriptionObserver<TOut, TError> {
	subscribe: (innerOpts: SubscriptionObserver<TOut, TError>) => void;
	enabled: boolean;
	queryKey: unknown;
}

type SubscriptionStatus = "idle" | "pending" | "success" | "error";

export function useSubscription<TOut, TError>(
	options: SubscriptionOptions<TOut, TError>,
): {
	data: Accessor<TOut | undefined>;
	error: Accessor<TError | undefined>;
	status: Accessor<SubscriptionStatus>;
} {
	const [data, setData] = createSignal<TOut | undefined>(undefined);
	const [error, setError] = createSignal<TError | undefined>(undefined);
	const [status, setStatus] = createSignal<SubscriptionStatus>("idle");

	options.subscribe({
		onStarted() {
			options.onStarted?.();
			setStatus("pending");
			setError(undefined);
		},
		onData(value) {
			options.onData?.(value);
			setStatus("pending");
			setData(() => value);
			setError(undefined);
		},
		onComplete() {
			options.onComplete?.();
			setStatus("success");
		},
		onError(err) {
			setError(() => err);
			options.onError?.(err);
			setStatus("error");
		},
	});

	return { data, error, status };
}
