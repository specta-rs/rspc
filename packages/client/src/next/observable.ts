export function observable<T>(
	cb: (subscriber: { next: (value: T) => void; complete(): void }) => void,
) {
	let callbacks: Array<(v: T) => void> = [];
	let completeCallbacks: Array<() => void> = [];
	let done = false;

	cb({
		next: (v) => {
			if (done) return;
			callbacks.forEach((cb) => cb(v));
		},
		complete: () => {
			if (done) return;
			done = true;
			completeCallbacks.forEach((cb) => cb());
		},
	});

	return {
		subscribe(cb: (v: T) => void) {
			if (done) return Promise.resolve();

			callbacks.push(cb);
			return new Promise<void>((res) => {
				completeCallbacks.push(() => res());
			});
		},
		get done() {
			return done;
		},
	};
}

export type Observable<T> = ReturnType<typeof observable<T>>;
