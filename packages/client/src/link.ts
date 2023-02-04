/**
 * A map of data that can be used by links to store metadata about the current operation.
 * This allows links to communicate with each other.
 *
 * @internal
 */
export type OperationContext = Record<string, unknown>;

/**
 * TODO
 *
 * @internal
 */
export interface Operation {
  // id: number; // TODO: Optional on being a subscription?
  type: "query" | "mutation" | "subscription";
  input: unknown;
  path: string;
  context: OperationContext;
}

/**
 * Link flag is a marker used to indicate a link has a special behavior.
 * This is used to make certain runtime errors impossible by catching them in the type system.
 *
 * @internal
 */
export type LinkFlag = "subscriptionsUnsupported" | "terminatedLink" | "built";

/**
 * An object representing a set of enabled link flags.
 *
 * @internal
 */
export type LinkFlags = { [K in LinkFlag]?: true };

/**
 * Takes in two sets of flags and returns a new set of flags that is the union of the input.
 * This exists because it takes into account the fact that null is a valid flag.
 *
 * @internal
 */
export type JoinLinkFlags<
  TFlag extends LinkFlags,
  TNewFlag extends LinkFlag
> = TFlag & { [K in TNewFlag]: true };

/**
 * Takes in a set of flags and a union of flags. It will return true if ALL of the flags are enabled.
 *
 * @internal
 */
export type HasLinkFlags<
  TFlags extends LinkFlags,
  TFlag extends LinkFlag
> = TFlags extends Record<TFlag, any> ? true : false;

/**
 * Takes in a set of flags and a union of flags. It will return true if ANY of the flags are enabled.
 *
 * @internal
 */
export type HasAnyLinkFlags<
  TFlags extends LinkFlags,
  TFlag extends LinkFlag
> = {
  [K in TFlag]: TFlags[K] extends true ? true : false;
}[TFlag] extends false
  ? false
  : true;
