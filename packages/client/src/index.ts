export * from "./error";
export * from "./typescript";
export * from "./links";
export * from "./internals/observable";
export * from "./client";
export * from "./client2";
export * from "./bindings";
export * from "./fetchLink";
export * from "./interop";

// TODO: Break out following

// TODO: Rename
export type ObservableLiteTeardownFn = (() => void) | void;

export interface ObserverLite {
  next(data: any): void;
  //   error(): void;
  //   complete(): void;
}

// TODO: Rename
export interface ObservableLite {
  subscribe(observer: ObserverLite): ObservableLiteTeardownFn;
}

// observer: ObservableLite

// TODO: Unit test this
export function observableToPromise(observable: ObservableLite) {
  let abort: () => void;
  const promise = new Promise((resolve, reject) => {
    let isDone = false;
    function onDone() {
      if (isDone) {
        return;
      }
      isDone = true;
      //   reject(new ObservableAbortError("This operation was aborted.")); // TODO
      if (unsubscribe) unsubscribe(); // TODO: Should this be before the `early return`. What are the semantics here?
    }
    const unsubscribe = observable.subscribe({
      next(data) {
        isDone = true;
        resolve(data);
        onDone();
      },
      //   error(data) {
      //     isDone = true;
      //     reject(data);
      //     onDone();
      //   },
      //   complete() {
      //     isDone = true;
      //     onDone();
      //   },
    });
    abort = onDone;
  });
  return {
    promise,
    abort: abort!,
  };
}
