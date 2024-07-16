import type { Procedures } from "@rspc/client";
import type { Context } from "@rspc/query-core";
import { getContext, setContext } from "svelte";

const _contextKey = "$$_rspcClient";

/** Retrieves a Client from Svelte's context */
export const getRspcClientContext = <
	P extends Procedures,
>(): Context<P> | null => {
	const ctx = getContext(_contextKey) ?? null;
	return ctx as Context<P> | null;
};

/** Sets a Client on Svelte's context */
export const setRspcClientContext = (ctx: Context<any>) =>
	setContext(_contextKey, ctx);
