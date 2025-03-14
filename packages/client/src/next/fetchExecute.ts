import { observable } from "./observable";
import { ExecuteArgs, ExecuteFn } from "./types";

export const fetchExecute = (
	config: { url: string },
	args: ExecuteArgs,
): ReturnType<ExecuteFn> => {
	if (args.type === "subscription")
		throw new Error("Subscriptions are not possible with the `fetch` executor");

	const url = new URL(`${config.url}/${args.path}`);

	let promise;
	if (args.type === "query") {
		url.search = new URLSearchParams(
			args.input === undefined ? {} : { input: JSON.stringify(args.input) },
		).toString();

		promise = fetch(url.toString(), {
			method: "GET",
			headers: {
				Accept: "application/json",
			},
		});
	} else {
		promise = fetch(url, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Accept: "application/json",
			},
			body: JSON.stringify(args.input),
		});
	}

	return observable((subscriber) => {
		promise
			.then(async (r) => {
				if (r.status === 200) {
					const json: {
						id: number | null;
						result: { type: "response"; data: any };
					} = await r.json();

					if (json.result.type === "response") {
						subscriber.next({ type: "data", value: json.result.data });
					}
				}
			})
			.finally(() => subscriber.complete());
	});
};
