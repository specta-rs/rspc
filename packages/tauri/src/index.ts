import { OperationType, Transport } from "@rspc/client";
import { ProcedureKind } from "@rspc/client/next";

import { tauriExecute } from "./next";

export class TauriTransport implements Transport {
	clientSubscriptionCallback?: (id: string, value: any) => void;

	id = 0;

	async doRequest(
		operation: OperationType,
		key: string,
		data: any,
	): Promise<any> {
		return await new Promise((resolve, reject) => {
			if (operation === "subscription") resolve(undefined);

			let input;
			if (operation === "subscription") {
				input = data[1];
			} else {
				input = data;
			}

			const obs = tauriExecute({
				type: operation as ProcedureKind,
				path: key,
				input,
			});

			obs.subscribe({
				next: (value) => {
					if (operation === "subscription") {
						if (value.type === "data")
							this.clientSubscriptionCallback?.(data[0], value.value);
					} else {
						if (value.type === "data") {
							resolve(value.value);
						}
					}
				},
				error(error) {
					reject(error);
				},
			});
		});
	}
}
