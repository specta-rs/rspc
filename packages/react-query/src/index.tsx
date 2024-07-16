import * as rspc from "@rspc/client";

import { createHooks } from "./hooks";
import type { ReactQueryProceduresProxy, ReactQueryProxy } from "./types";

export function createReactQueryProxy<
	P extends rspc.Procedures,
>(): ReactQueryProxy<P> {
	const hooks = createHooks();

	return new Proxy({} as any, {
		get(_, key) {
			if (typeof key !== "string") return;

			if (key in hooks) return hooks[key as keyof typeof hooks];

			return rspc.createProceduresProxy<ReactQueryProceduresProxy<P>>(
				({ args, path }) => {
					const operation = path.pop();

					if (
						operation === "useQuery" ||
						operation === "useMutation" ||
						operation === "useSubscription"
					)
						return hooks[operation](path, ...args);
				},
				[key],
			);
		},
	});
}
