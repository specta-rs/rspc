import {
  HasAnyLinkFlags,
  HasLinkFlags,
  inferProcedureResult,
  JoinLinkFlags,
  Link,
  LinkFlags,
  Operation,
  ProceduresDef,
  SubscriptionOptions,
  _inferProcedureHandlerInput,
} from ".";

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

/**
 * @internal
 */
export type InitRspcInnerArgs = InitRspcOpts & {
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
      return exec(opts, {
        id: 0,
        type: "query",
        path: keyAndInput[0] as any,
        input: keyAndInput[1] as any,
        context: {},
      });
    },
    mutate<K extends T["mutations"]["key"] & string>(
      keyAndInput: [
        key: K,
        ...input: _inferProcedureHandlerInput<T, "queries", K>
      ]
    ): Promise<inferProcedureResult<T, "mutations", K>> {
      return exec(opts, {
        id: 0,
        type: "mutation",
        path: keyAndInput[0] as any,
        input: keyAndInput[1] as any,
        context: {},
      });
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

      return () => {
        // TODO: Remove subscription
      };
    },
  } satisfies Rspc<T, {} /* TODO: Should be default? */> as any;
}

function exec(opts: InitRspcInnerArgs, op: Operation): Promise<any> {
  // TODO: Handle executing with multiple links in the observable lite system.
  // TODO: Move this exec login into the `fakeObservable` function
  const resp = opts.links[0]!({
    op,
    next() {
      throw new Error("TODO: Probally unreachable"); // TODO: Deal with this
    },
  });

  // TODO: Expose `.abort` to the end user like tRPC does
  return resp.exec().promise.then((v) => v.result.data);
}

// TODO: export function getQueryKey() {}
