/**
 * TODO
 *
 * @internal
 */
export type AbortFn = () => void;

/**
 * TODO
 *
 * @internal
 */
export type PromiseAndCancel<TValue> = {
  promise: Promise<TValue>;
  abort: AbortFn;
};

/**
 * TODO
 *
 * @internal
 */
export type FakeObservable = { exec: () => PromiseAndCancel<any> };
