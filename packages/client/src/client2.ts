// TODO: Split this stuff into a `client.ts` and `link.ts` files

import { ObservableLite } from ".";
import { ProceduresDef } from "./bindings";

// TODO: Rename middleware to `link` maybe? Or transformer?

/**
 * @internal
 *
 * TODO: Rename
 */
export type OperationContext2 = Record<string, unknown>;

/**
 * @internal
 *
 * TODO: Rename
 */
export type Operation2 = {
  // id: number; // TODO: Optional on being a subscription?
  type: "query" | "mutation" | "subscription";
  input: unknown;
  path: string;
  context: OperationContext2;
};

/**
 * TODO
 *
 * @internal
 */
export interface MiddlewareOperation<T extends ProceduresDef>
  extends Operation2 {
  // next();
}

/**
 * TODO
 *
 * @internal
 */
export type Middleware<T extends ProceduresDef, TT, TFlag extends Flag> = (
  p: MiddlewareOperation<T>
) => MiddlewareResp<TT, TFlag>;

/**
 * TODO
 * TODO: Should this be marked as internal because it will be required to make a custom link.
 *
 * @internal
 */
export type MiddlewareResp<T, TFlag extends Flag = null> = ObservableLite;

/**
 * A flag is a string that is used to indicate some special behavior of a link chain.
 * This is used to make certain runtime errors impossible by catching them in the type system.
 *
 * @internal
 */
export type Flag = "subscriptionsUnsupported" | "terminatedLink" | null;

/**
 * Takes in two sets of flags and returns a new set of flags that is the union of the input.
 * This exists because it takes into account the fact that null is a valid flag.
 *
 * @internal
 */
type JoinFlags<TFlag extends Flag, TNewFlag extends Flag> = Exclude<
  TFlag | TNewFlag,
  null
> extends never
  ? null
  : Exclude<TFlag | TNewFlag, null>;

/**
 * TODO
 *
 * @internal
 */
export type Rspc<T extends ProceduresDef, TFlag extends Flag = null> = {
  query<K extends T extends { key: string } ? T["key"] : never>(key: K): void;
} & (TFlag extends "terminatedLink"
  ? {}
  : {
      use<TT extends ProceduresDef, TNewFlag extends Flag>(
        mw: Middleware<T, TT, TNewFlag>
      ): Rspc<TT, JoinFlags<TFlag, TNewFlag>>;
    }) &
  (TFlag extends "subscriptionsUnsupported" ? {} : { subscribe(): void });

function initRspcInner<
  T extends ProceduresDef,
  TFlag extends Flag = null
>(): Rspc<T, TFlag> {
  return {
    // @ts-expect-error: TODO: Fix this. It's because of the discriminated union.
    use<TT extends ProceduresDef, TNewFlag extends Flag>(
      mw: Middleware<T, TT, TNewFlag>
    ): Rspc<TT, JoinFlags<TFlag, TNewFlag>> {
      return initRspcInner<TT, JoinFlags<TFlag, TNewFlag>>();
    },
    query<K extends T extends { key: string } ? T["key"] : never>(key: K) {},
    subscribe() {},
  } satisfies Rspc<T, null> as any;
}

/**
 * TODO
 */
export interface InitRspcOpts {
  /**
   * TODO
   */
  // onError(); // TODO: Make this work with links!
}

/**
 * TODO
 */
export function initRspc<T extends ProceduresDef>(
  opts?: InitRspcOpts
): Rspc<T> {
  return initRspcInner<T>();
}
