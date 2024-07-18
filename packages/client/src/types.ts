export type JoinPath<
	TPath extends string,
	TNext extends string,
> = TPath extends "" ? TNext : `${TPath}.${TNext}`;

export type ProcedureKind = "query" | "mutation" | "subscription";

export type Procedure = {
	kind: ProcedureKind;
	input: unknown;
	result: unknown;
	error: unknown;
};

export type Procedures = {
	[K in string]: Procedure | Procedures;
};

export type Result<Ok, Err> =
	| { status: "ok"; data: Ok }
	| { status: "err"; error: Err };

export type ProcedureResult<P extends Procedure> = Result<
	P["result"],
	P["error"]
>;
