import {
  HasAnyLinkFlags,
  HasLinkFlags,
  inferProcedureResult,
  inferQueryResult,
  JoinLinkFlags,
  LinkFlags,
  Observable,
  observable,
  observableToPromise,
  Operation,
  ProceduresDef,
  SubscriptionOptions,
  _inferProcedureHandlerInput,
} from ".";

/**
 * TODO
 *
 * @internal
 */
// TODO: Probs remove generic from this cause it's not really needed?
export interface LinkOperation<T extends ProceduresDef> {
  op: Operation;
  next(op: LinkOperation<T>): void; // TODO: return type
}

/**
 * TODO
 *
 * @internal
 */
export type Link<
  T extends ProceduresDef,
  TT extends ProceduresDef = T,
  TFlags extends LinkFlags = {}
> = (p: LinkOperation<T>) => LinkResponse<TT, TFlags>;

/**
 * TODO
 * TODO: Should this be marked as internal because it will be required to make a custom link.
 *
 * @internal
 */
export type LinkResponse<
  T extends ProceduresDef,
  TFlags extends LinkFlags
> = Observable<any, any>; // TODO: Replace any's???

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
  return initRspcInner<T>({ ...opts, links: [] });
}

// TODO: Utils
type And<T, U> = T extends true ? (U extends true ? true : false) : false;
type Or<T, U> = T extends true ? true : U extends true ? true : false;
type Not<T> = T extends true ? false : true;

// https://github.com/microsoft/TypeScript/issues/27024#issuecomment-421529650
// type Equals<X, Y> = (<T>() => T extends X ? 1 : 2) extends <T>() => T extends Y
//   ? 1
//   : 2
//   ? true
//   : false;

type LinkArg<
  T extends ProceduresDef,
  TT extends ProceduresDef,
  TFlags extends LinkFlags,
  TNewFlags extends LinkFlags = {}
> = And<
  HasLinkFlags<TFlags, "built">,
  Not<HasLinkFlags<TFlags, "subscriptionsUnsupported">>
> extends true
  ? HasLinkFlags<TNewFlags, "subscriptionsUnsupported"> extends true
    ? "You must provide a link which supports subscriptions!" // TODO: This doesn't work with the args hack below so...
    : Link<T, TT, TNewFlags>
  : Link<T, TT, TNewFlags>;

type UseFn<T extends ProceduresDef, TFlags extends LinkFlags> = HasAnyLinkFlags<
  TFlags,
  "terminatedLink"
> extends false
  ? {
      use<TT extends ProceduresDef, TNewFlags extends LinkFlags>(
        link: LinkArg<T, TT, TFlags, TNewFlags>
        // TODO: Can this be done in a different way for a better error message?
        //   ...cannotUseTypeMappingLinkAfterCallingExportMethod: HasLinkFlags<
        //     TFlags,
        //     "built"
        //   > extends true
        //     ? Equals<T, TT> extends true
        //       ? []
        //       : [never]
        //     : []
      ): Rspc<TT, TFlags & TNewFlags>;
    }
  : {};

type BuildFn<T extends ProceduresDef, TFlags extends LinkFlags> = Or<
  HasAnyLinkFlags<TFlags, "built">,
  HasAnyLinkFlags<TFlags, "terminatedLink">
> extends false
  ? {
      // TODO: This is marked as unstable because it's not properly typesafe. Will be stablised in the future.
      unstable_build<TSupportSubscriptions extends boolean>(opts: {
        supportsSubscriptions: TSupportSubscriptions;
      }): Rspc<
        T,
        TFlags & {
          built: true;
        } & (TSupportSubscriptions extends false
            ? { subscriptionsUnsupported: true }
            : {})
      >;
    }
  : {};

type OperationFns<
  T extends ProceduresDef,
  TFlags extends LinkFlags
> = HasAnyLinkFlags<TFlags, "terminatedLink" | "built"> extends true
  ? {
      query<K extends T["queries"]["key"] & string>(
        keyAndInput: [
          key: K,
          ...input: _inferProcedureHandlerInput<T, "queries", K>
        ]
      ): Promise<inferProcedureResult<T, "queries", K>>;
      mutate<K extends T["mutations"]["key"] & string>(
        keyAndInput: [
          key: K,
          ...input: _inferProcedureHandlerInput<T, "queries", K>
        ]
      ): Promise<inferProcedureResult<T, "mutations", K>>;
    } & (HasAnyLinkFlags<TFlags, "subscriptionsUnsupported"> extends true
      ? {}
      : {
          subscribe<
            K extends T["subscriptions"]["key"] & string,
            TData = inferProcedureResult<T, "subscriptions", K>
          >(
            keyAndInput: [
              K,
              _inferProcedureHandlerInput<T, "subscriptions", K>
            ],
            opts: SubscriptionOptions<TData>
          ): () => void;
          /**
           * @deprecated use `.subscribe` instead. This will be removed in a future release.
           */
          addSubscription<
            K extends T["subscriptions"]["key"] & string,
            TData = inferProcedureResult<T, "subscriptions", K>
          >(
            keyAndInput: [
              K,
              _inferProcedureHandlerInput<T, "subscriptions", K>
            ],
            opts: SubscriptionOptions<TData>
          ): () => void;
        })
  : {};

/**
 * The type of the rspc instance. This type is what powers all of the advanced type checking.
 *
 * @internal
 */
export type Rspc<
  T extends ProceduresDef,
  TFlags extends LinkFlags = {}
> = UseFn<T, TFlags> & OperationFns<T, TFlags> & BuildFn<T, TFlags>;

type InitRspcInnerArgs = InitRspcOpts & {
  links: Link<any, any, any>[];
};

// TODO: Expose AbortController through vanilla client
function initRspcInner<T extends ProceduresDef, TFlag extends LinkFlags = {}>(
  opts: InitRspcInnerArgs
): Rspc<T, TFlag> {
  // TODO: Fix this cringe internal type safety of this function.
  // TODO: The implication of this setup is that `Command + Click` on a method externally takes you to the type not the method. Try and fix this!
  return {
    // @ts-expect-error: TODO: Fix this. It's because of the discriminated union.
    use<TT extends ProceduresDef, TNewFlag extends Flag>(
      link: Link<T, TT, TNewFlag>
    ): Rspc<TT, JoinLinkFlags<TFlag, TNewFlag>> {
      opts.links.push(link);
      return initRspcInner<TT, JoinLinkFlags<TFlag, TNewFlag>>(opts);
    },
    query<K extends T["queries"]["key"] & string>(
      keyAndInput: [
        key: K,
        ...input: _inferProcedureHandlerInput<T, "queries", K>
      ]
    ): Promise<inferProcedureResult<T, "queries", K>> {
      const observable = exec(opts, {
        op: {
          type: "query",
          path: keyAndInput[0] as any,
          input: keyAndInput[1] as any,
          context: {},
        },
        next() {
          throw new Error("TODO: Probally unreachable"); // TODO: Deal with this
        },
      });

      const { promise, abort } = observableToPromise(observable);
      // TODO: Expose `abort` function to user if they want it -> Maybe an arg, idk how tRPC do it?

      // TODO: Should we expose `v.context`???
      // @ts-expect-error // TODO: Fix type error at some point
      return promise.then((v) => v.result.data);
    },
    mutate<K extends T["mutations"]["key"] & string>(
      keyAndInput: [
        key: K,
        ...input: _inferProcedureHandlerInput<T, "queries", K>
      ]
    ): Promise<inferProcedureResult<T, "mutations", K>> {
      const observable = exec(opts, {
        op: {
          type: "mutation",
          path: keyAndInput[0] as any,
          input: keyAndInput[1] as any,
          context: {},
        },
        next() {
          throw new Error("TODO: Probally unreachable"); // TODO: Deal with this
        },
      });

      const { promise, abort } = observableToPromise(observable);
      // TODO: Expose `abort` function to user if they want it -> Maybe an arg, idk how tRPC do it?

      // TODO: Should we expose `v.context`???
      // @ts-expect-error // TODO: Fix type error at some point
      return promise.then((v) => v.result.data);
    },
    subscribe<
      K extends T["subscriptions"]["key"] & string,
      TData = inferProcedureResult<T, "subscriptions", K>
    >(
      keyAndInput: [K, _inferProcedureHandlerInput<T, "subscriptions", K>],
      opts: SubscriptionOptions<TData>
    ): () => void {
      // TODO: Support for subscriptions!
      throw new Error(
        "TODO: Subscriptions are not yet supported on the alpha client!"
      );
    },
  } satisfies Rspc<T, {} /* TODO: Should be default? */> as any;
}

const exec = (opts: InitRspcInnerArgs, initialOp: LinkOperation<any>) =>
  observable((observer) => {
    function execute(index = 0, op = initialOp) {
      const next = opts.links[index];
      if (!next) {
        throw new Error(
          "No more links to execute - did you forget to add a terminating link?"
        );
      }

      const subscription = next({
        op: op.op,
        next(nextOp) {
          const nextObserver = execute(index + 1, nextOp);

          return nextObserver;
        },
      });
      return subscription;
    }

    const obs$ = execute();
    return obs$.subscribe(observer);
  });

// TODO: export function getQueryKey() {}
