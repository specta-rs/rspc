import * as rspc from "@rspc/client";

import { createHooks } from "./hooks";
import type { SolidQueryProceduresProxy, SolidQueryProxy } from "./types";

export function createSolidQueryProxy<
	P extends rspc.Procedures,
>(): SolidQueryProxy<P> {
	const hooks = createHooks();

	return new Proxy({} as any, {
		get(_, key) {
			if (typeof key !== "string") return;

			if (key in hooks) return hooks[key as keyof typeof hooks];

			return rspc.createProceduresProxy<SolidQueryProceduresProxy<P>>(
				({ args, path }) => {
					const operation = path.pop();

					if (
						operation === "createQuery" ||
						operation === "createMutation" ||
						operation === "createSubscription"
					)
						return hooks[operation](path, ...args);
				},
				[key],
			);
		},
	});
}
