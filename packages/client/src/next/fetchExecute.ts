import { observable } from "./observable";
import { ExecuteArgs, ExecuteFn } from "./types";

type BatchLoader = {
	data: [string, any][];
	callbacks: ((v: [number, any]) => void)[];
};

const batchLoaders = {
	query: null as null | BatchLoader,
	mutation: null as null | BatchLoader,
};

// - batch: false, stream: false
// no batching or streaming. execution happens individually and no data is streamed.
//
// - batch: true, stream: false
// executions are batched by their type, returned data is same as above.
//
// - batch: false, stream: true - not implemented yet
// execution happens individually but streamed queries/mutations are buffered on client,
// and subscriptions are supported.
//
// - batch: true, stream: true
// executions are batched by their type, streaming behaviour is same as above,
// with results potentially returning out of order

export const fetchExecute = (
	config: { url: string; batch?: boolean; stream?: boolean },
	args: ExecuteArgs,
): ReturnType<ExecuteFn> => {
	if (args.type === "subscription")
		throw new Error("Subscriptions are not possible with the `fetch` executor");

	if (!config.batch) {
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
						subscriber.next({ type: "data", value: await r.json() });
						subscriber.complete();
					}
				})
				.catch((e) => {
					subscriber.error(e.toString());
				});
		});
	} else {
		const type = args.type;
		let batchLoader = batchLoaders[type];

		if (!batchLoader) {
			batchLoader = batchLoaders[type] = {
				data: [],
				callbacks: [],
			};

			setTimeout(async () => {
				if (!batchLoader) return;
				batchLoaders[type] = null;

				const resp = await fetch(config.url, {
					method: "POST",
					body: JSON.stringify(batchLoader.data),
					headers: {
						"Content-Type": "application/json",
						...(config.stream ? { "rspc-batch-mode": "stream" } : {}),
					},
				});

				if (!config.stream) {
					const items: [number, any][] = await resp.json();

					batchLoader.callbacks.forEach((v, i) => {
						v(items[i]);
					});
				} else {
					if (!resp.body) throw new Error("response has no body??");

					const reader = resp.body.getReader();
					const decoder = new TextDecoder();

					while (true) {
						const { done, value } = await reader.read();
						if (done) break;

						const line = decoder.decode(value);

						const regex = /(\d+):(\[.*?\])/;
						const match = line.match(regex);
						if (!match) throw new Error("invalid stream content!");

						const index = parseInt(match[1]);
						const [status, data] = JSON.parse(match[2]);

						batchLoader.callbacks[index]?.([status, data]);
					}
				}
			}, 1);
		}

		batchLoader.data.push([
			args.path,
			args.input === undefined ? null : args.input,
		]);

		return observable((observer) => {
			batchLoader.callbacks.push(([status, data]) => {
				if (status === 200) {
					observer.next({ type: "data", value: data });
					observer.complete();
				} else {
					observer.error(data);
				}
			});
		});
	}
};
