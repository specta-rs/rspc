export interface Observer<TData, TError> {
	next(value: TData): void;
	error(error: TError): void;
	complete(): void;
}

export function observable<TData, TError>(
	cb: (observer: Observer<TData, TError>) => void,
) {
	let isDone = false;

	return {
		subscribe(observer: Partial<Observer<TData, TError>>) {
			cb({
				next: (value) => {
					if (isDone) return;
					observer.next?.(value);
				},
				error: (error) => {
					if (isDone) return;
					isDone = true;
					observer.error?.(error);
				},
				complete: () => {
					if (isDone) return;
					isDone = true;
					observer.complete?.();
				},
			});
		},
		get done() {
			return isDone;
		},
	};
}

export type Observable<TData, TError> = ReturnType<
	typeof observable<TData, TError>
>;
