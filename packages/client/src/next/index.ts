import { UntypedClient } from "./UntypedClient";
import type {
	ProcedureResult,
	ProcedureKind as ProcedureKind,
	SubscriptionObserver,
	ExecuteFn,
	Procedure,
} from "./types";

export * from "./types";
export { Observable, observable } from "./UntypedClient";

export type ProcedureWithKind<V extends ProcedureKind> = Omit<
	Procedure,
	"kind"
> & { kind: V };

export type Procedures = {
	[K in string]: Procedure | Procedures;
};

type Unsubscribable = { unsubscribe: () => void };

export type VoidIfInputNull<
	P extends Procedure,
	Else = P["input"],
> = P["input"] extends null ? void : Else;

type Resolver<P extends Procedure> = (
	input: VoidIfInputNull<P>,
) => Promise<ProcedureResult<P>>;

type SubscriptionResolver<P extends Procedure> = (
	input: VoidIfInputNull<P>,
	opts?: Partial<SubscriptionObserver<P["output"], P["error"]>>,
) => Unsubscribable;

export type ProcedureProxyMethods<P extends Procedure> =
	P["kind"] extends "query"
		? { query: Resolver<P> }
		: P["kind"] extends "mutation"
			? { mutate: Resolver<P> }
			: P["kind"] extends "subscription"
				? { subscribe: SubscriptionResolver<P> }
				: never;

type ClientProceduresProxy<P extends Procedures> = {
	[K in keyof P]: P[K] extends Procedure
		? ProcedureProxyMethods<P[K]>
		: P[K] extends Procedures
			? ClientProceduresProxy<P[K]>
			: never;
};

export type Client<P extends Procedures> = ClientProceduresProxy<P>;

const noop = () => {
	// noop
};

interface ProxyCallbackOptions {
	path: string[];
	args: any[];
}
type ProxyCallback = (opts: ProxyCallbackOptions) => unknown;

const clientMethodMap = {
	query: "query",
	mutate: "mutation",
	subscribe: "subscription",
} as const;

export function createProceduresProxy<T>(
	callback: ProxyCallback,
	path: string[] = [],
): T {
	return new Proxy(noop, {
		get(_, key) {
			if (typeof key !== "string") return;

			return createProceduresProxy(callback, [...path, key]);
		},
		apply(_1, _2, args) {
			return callback({ args, path });
		},
	}) as T;
}

export function createClient<P extends Procedures>(
	execute: ExecuteFn,
): Client<P> {
	const client = new UntypedClient(execute);

	return createProceduresProxy<Client<P>>(({ args, path }) => {
		const procedureType =
			clientMethodMap[path.pop() as keyof typeof clientMethodMap];

		const pathString = path.join(".");

		return (client[procedureType] as any)(pathString, ...args);
	});
}

export function getQueryKey(
	path: string,
	input: unknown,
): [string] | [string, unknown] {
	return input === undefined ? [path] : [path, input];
}

export function traverseClient<P extends Procedure>(
	client: Client<any>,
	path: string[],
): ProcedureProxyMethods<P> {
	let ret: ClientProceduresProxy<Procedures> = client;

	for (const segment of path) {
		ret = ret[segment];
	}

	return ret as any;
}
