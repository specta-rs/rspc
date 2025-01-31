// This file was generated by [rspc](https://github.com/specta-rs/rspc). Do not edit this file manually.

export type ProceduresLegacy = { queries: { key: "echo"; input: string; result: string } | { key: "error"; input: null; result: string } | { key: "transformMe"; input: null; result: string } | { key: "version"; input: null; result: string }; mutations: { key: "sendMsg"; input: string; result: string }; subscriptions: { key: "pings"; input: null; result: string } }

export type Procedures = {
	echo: { kind: "query", input: string, output: string, error: unknown },
	error: { kind: "query", input: null, output: string, error: unknown },
	pings: { kind: "subscription", input: null, output: string, error: unknown },
	sendMsg: { kind: "mutation", input: string, output: string, error: unknown },
	transformMe: { kind: "query", input: null, output: string, error: unknown },
	version: { kind: "query", input: null, output: string, error: unknown },
}