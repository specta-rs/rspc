import {
  ClientArgs as TRPCClientArgs,
  TRPCClientRuntime,
  OperationContext,
  OperationLink,
  DataTransformer,
  Operation,
  share,
  inferObservableValue,
  observableToPromise,
  RSPCError,
  TRPCRequestOptions,
  TRPCSubscriptionObserver,
  TRPCLink,
} from ".";
import { createChain } from "./links/internals/createChain";
import { inferProcedures, ProcedureDef, ProceduresDef } from "./typescript";

// UTILITIES
//
type Queries<Procs extends ProceduresDef> = Procs["queries"];
type Query<
  Procs extends ProceduresDef,
  K extends Queries<Procs>["key"]
> = GetProcedure<Queries<Procs>, K>;
type Mutations<Procs extends ProceduresDef> = Procs["mutations"];
type Mutation<
  Procs extends ProceduresDef,
  K extends Mutations<Procs>["key"]
> = GetProcedure<Mutations<Procs>, K>;
type Subscriptions<Proces extends ProceduresDef> = Proces["subscriptions"];
type Subscription<
  Procs extends ProceduresDef,
  K extends Subscriptions<Procs>["key"]
> = GetProcedure<Subscriptions<Procs>, K>;

function getTransformer(opts: ClientArgs<any>): DataTransformer {
  if (!opts.transformer)
    return {
      serialize: (data) => data,
      deserialize: (data) => data,
    };
  if ("input" in opts.transformer)
    return {
      serialize: opts.transformer.input.serialize,
      deserialize: opts.transformer.output.deserialize,
    };
  return opts.transformer;
}

type GetProcedure<
  P extends ProcedureDef,
  K extends P["key"] & string
> = Extract<P, { key: K }>;

type TupleCond<T, Cond> = T extends Cond ? [] : [T];

// I think TS will only map over a union type if you use a conditional - @brendonovich
type ProcedureKeyTuple<P extends ProcedureDef> = P extends ProcedureDef
  ? [key: P["key"], ...input: TupleCond<P["input"], null>]
  : never;

type FilterFn<I extends ProceduresDef> = (i: FilterData<I>) => FilterData<any>;

type ApplyFilter<
  TProcs extends ProceduresDef,
  TArgs extends ClientArgs<TProcs>
> = TArgs["filter"] extends FilterFn<TProcs>
  ? ReturnType<TArgs["filter"]> extends FilterData<infer P>
    ? P
    : never
  : TProcs;

// CLIENT

interface ClientArgs<TProcs extends ProceduresDef> extends TRPCClientArgs {
  filter?: FilterFn<TProcs>;
}

class Client<TProcs extends ProceduresDef, TArgs extends ClientArgs<TProcs>> {
  requestId = 0;
  links?: OperationLink<any>[];
  runtime: TRPCClientRuntime;

  constructor(args: TArgs) {
    this.runtime = {
      transformer: getTransformer(args),
      onError: args.onError,
    };

    if (args.links) this.$setLinks(args.links);
  }

  private $request<TProc extends ProcedureDef>({
    type,
    keyAndInput,
    context = {},
  }: {
    type: Operation["type"];
    keyAndInput: ProcedureKeyTuple<TProc>;
    context?: OperationContext;
  }) {
    return createChain<inferProcedures<any>, TProc["input"], TProc["result"]>({
      links: this.links as OperationLink<any, any, any>[],
      op: {
        id: ++this.requestId,
        type,
        path: keyAndInput[0],
        input: keyAndInput[1],
        context,
      },
    }).pipe(share());
  }

  private requestAsPromise<TProc extends ProcedureDef>(opts: {
    type: Operation["type"];
    keyAndInput: ProcedureKeyTuple<TProc>;
    context?: OperationContext;
    signal?: AbortSignal;
  }) {
    const req$ = this.$request(opts);
    type TValue = inferObservableValue<typeof req$>;
    const { promise, abort } = observableToPromise<TValue>(req$);

    return new Promise<TProc["result"]>((resolve, reject) => {
      opts.signal?.addEventListener("abort", abort);

      promise
        .then((envelope) => {
          resolve(envelope.result.data);
        })
        .catch((err) => {
          reject(RSPCError.from(err));
        });
    });
  }

  query<K extends Queries<ApplyFilter<TProcs, TArgs>>["key"] & string>(
    keyAndInput: ProcedureKeyTuple<Query<ApplyFilter<TProcs, TArgs>, K>>,
    opts?: TRPCRequestOptions
  ) {
    return this.requestAsPromise({
      type: "query",
      keyAndInput,
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  mutation<K extends Mutations<ApplyFilter<TProcs, TArgs>>["key"] & string>(
    keyAndInput: ProcedureKeyTuple<Mutation<ApplyFilter<TProcs, TArgs>, K>>,
    opts?: TRPCRequestOptions
  ) {
    return this.requestAsPromise({
      type: "mutation",
      keyAndInput,
      context: opts?.context,
      signal: opts?.signal,
    });
  }

  subscription<
    K extends Subscriptions<ApplyFilter<TProcs, TArgs>>["key"] & string
  >(
    keyAndInput: ProcedureKeyTuple<Subscription<ApplyFilter<TProcs, TArgs>, K>>,
    opts: TRPCRequestOptions &
      Partial<
        TRPCSubscriptionObserver<
          Subscription<ApplyFilter<TProcs, TArgs>, K>["result"],
          RSPCError
        >
      >
  ) {
    const observable$ = this.$request({
      type: "subscription",
      keyAndInput,
      context: opts?.context,
    });
    return observable$.subscribe({
      next(envelope) {
        if (envelope.result.type === "started") {
          opts.onStarted?.();
        } else if (envelope.result.type === "stopped") {
          opts.onStopped?.();
        } else {
          opts.onData?.(envelope.result.data);
        }
      },
      error(err) {
        opts.onError?.(err);
      },
      complete() {
        opts.onComplete?.();
      },
    });
  }

  $setLinks(links: TRPCLink<any>[]) {
    this.links = links.map((link) => link(this.runtime));
  }
}

export function rspcRoot<TProcs extends ProceduresDef>() {
  return {
    createClient<TArgs extends ClientArgs<TProcs>>(
      args: TArgs
    ): TArgs extends { links: any }
      ? Omit<Client<TProcs, TArgs>, "$setLinks">
      : Client<TProcs, TArgs> {
      return new Client<TProcs, TArgs>(args) as any;
    },
  };
}

// LIBRARY FILTER STUFF

interface FilterData<P extends ProceduresDef> {
  procedureKey:
    | [P["queries" | "mutations" | "subscriptions"]["key"]]
    | [P["queries" | "mutations" | "subscriptions"]["key"], any];
}

type StripLibraryArgsFromInput<T extends ProcedureDef> = T extends any
  ? T["input"] extends LibraryArgs<infer E>
    ? {
        key: T["key"];
        input: E;
        result: T["result"];
      }
    : never
  : never;

type LibraryProcedures<P extends ProcedureDef> = Exclude<
  Extract<P, { input: LibraryArgs<any> }>,
  { input: never }
>;

type AsLibraryProcedure<P extends ProcedureDef> = StripLibraryArgsFromInput<
  LibraryProcedures<P>
>;

type LibraryProceduresFilter<P extends ProceduresDef> = {
  queries: AsLibraryProcedure<Queries<P>>;
  mutations: AsLibraryProcedure<Mutations<P>>;
  subscriptions: AsLibraryProcedure<Subscriptions<P>>;
};

const LIBRARY_ID = "lmaoo";

export const libraryProceduresFilter = <P extends ProceduresDef>(
  p: FilterData<P>
): FilterData<LibraryProceduresFilter<P>> => {
  return {
    ...p,
    procedureKey: [
      p.procedureKey[0],
      { library_id: LIBRARY_ID, args: p.procedureKey[1] ?? null },
    ],
  };
};

// EXAMPLE

interface LibraryArgs<T> {
  library_id: string;
  arg: T;
}

type Procedures = {
  queries:
    | {
        key: "a";
        input: string;
        result: number;
      }
    | {
        key: "b";
        input: LibraryArgs<string>;
        result: string;
      };
  mutations: never;
  subscriptions: never;
};

const rspc = rspcRoot<Procedures>();

const regularClient = rspc.createClient({});
const libraryClient = rspc.createClient({
  filter: (d) => libraryProceduresFilter(d),
});

// const client = rspc.createClient({});

// Assuming vanilla is used without react context

// // @sd/client
// const rspc = initRspc<Procedures>();

// const api = rspc.createApi();
// const libraryApi = rspc.createApi({
// 		transform: (d) => libraryTransform(d)
// })

// // same configuraton as vanilla api
// const apiHooks = createReactHooks(rspc);
// const libraryApiHooks = createReactHooks(rspc, {
// 		t
