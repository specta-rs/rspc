import type { Observer } from "./full";

/**
 * @internal
 */
export type CancelFn = () => void;

/**
 * @internal
 */
export type PromiseAndCancel<TValue> = {
  promise: Promise<TValue>;
  cancel: CancelFn;
};

/**
 * TODO: Explain this
 *
 * @internal
 */
// TODO: Remove `any`
export function fakeObservable(
  handler: () => PromiseAndCancel<any>
): FakeObservable {
  return {
    exec: handler,
  };
}

// TODO: Rename and explain
export type FakeObservable = { exec: () => PromiseAndCancel<any> };

export function isFakeObservable(obj: any): obj is FakeObservable {
  return typeof obj.exec === "function" && typeof obj.subscribe === "undefined";
}

// TODO: Rename and explain
export type FullObservable = {
  exec: () => PromiseAndCancel<any>;
  subscribe: (opts: Observer<any, any>) => void;
};

// TODO: Make this work and include somewhere -> Maybe in full package export
export function isFullObservable(obj: any): obj is FullObservable {
  return typeof obj.exec === "function" && typeof obj.subscribe === "function";
}

// TODO: Rename and explain
export type NewObservable = FakeObservable | FullObservable;
