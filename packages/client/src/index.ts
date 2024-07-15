import {
	UntypedClient,
	type ProcedureVariant,
	type SubscriptionObserver,
} from "./UntypedClient";

type Procedure = {
	variant: ProcedureVariant;
	input: unknown;
	result: unknown;
	error: unknown;
};

export type Procedures = {
	[K in string]: Procedure | Procedures;
};

type Result<Ok, Err> =
	| { status: "ok"; data: Ok }
	| { status: "err"; error: Err };

type Unsubscribable = { unsubscribe: () => void };

type Resolver<P extends Procedure> = (
	input: P["input"],
) => Promise<Result<P["result"], P["error"]>>;

type SubscriptionResolver<P extends Procedure> = (
	input: P["input"],
	opts?: Partial<SubscriptionObserver<P["result"], P["error"]>>,
) => Unsubscribable;

type ProcedureProxyMethods<P extends Procedure> = P["variant"] extends "query"
	? { query: Resolver<P> }
	: P["variant"] extends "mutation"
		? { mutate: Resolver<P> }
		: P["variant"] extends "subscription"
			? { subscribe: SubscriptionResolver<P> }
			: never;

type ClientProceduresProxy<P extends Procedures> = {
	[K in keyof P]: P[K] extends Procedure
		? ProcedureProxyMethods<P[K]>
		: P[K] extends Procedures
			? ClientProceduresProxy<P[K]>
			: never;
};

type Client<P extends Procedures> = ClientProceduresProxy<P>;

const noop = () => {
	// noop
};

interface ProxyCallbackOptions {
	path: string[];
	args: unknown[];
}
type ProxyCallback = (opts: ProxyCallbackOptions) => unknown;

const clientMethodMap = {
	query: "query",
	mutate: "mutation",
	subscribe: "subscription",
} as const;

function createClientProxy<T>(callback: ProxyCallback, path: string[] = []): T {
	return new Proxy(noop, {
		get(_, key) {
			if (typeof key !== "string") return;

			return createClientProxy(callback, [...path, key]);
		},
		apply(_1, _2, args) {
			return callback({ args, path });
		},
	}) as T;
}

export function createClient<P extends Procedures>(): Client<P> {
	const client = new UntypedClient();

	return createClientProxy<Client<P>>(({ args, path }) => {
		const procedureType =
			clientMethodMap[path.pop() as keyof typeof clientMethodMap];

		const pathString = path.join(".");

		// biome-ignore lint/suspicious/noExplicitAny: type magic
		(client[procedureType] as any)(pathString, ...args);
	});
}
