import { Observable } from "./observable";

export type JoinPath<
	TPath extends string,
	TNext extends string,
> = TPath extends "" ? TNext : `${TPath}.${TNext}`;

export type ProcedureKind = "query" | "mutation" | "subscription";

export type Procedure = {
	kind: ProcedureKind;
	input: unknown;
	output: unknown;
	error: unknown;
};

export type Procedures = {
	[K in string]: Procedure | Procedures;
};

export type Result<Ok, Err> =
	| { status: "ok"; data: Ok }
	| { status: "err"; error: Err };

export type ProcedureResult<P extends Procedure> = Result<
	P["output"],
	P["error"]
>;

export interface SubscriptionObserver<TValue, TError> {
	onStarted: () => void;
	onData: (value: TValue) => void;
	onError: (err: TError) => void;
	onComplete: () => void;
}

export type ExecuteArgs = {
	type: ProcedureKind;
	path: string;
	input: unknown;
};
export type ExecuteFn = (args: ExecuteArgs) => Observable<ExeceuteData>;

export type ExeceuteData =
	| { type: "started" }
	| { type: "data"; value: unknown }
	| { type: "error"; error: unknown }
	| { type: "complete" };
