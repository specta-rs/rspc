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

export type GetProcedure<
  P extends ProcedureDef,
  K extends P["key"] & string
> = Extract<P, { key: K }>;

type TupleCond<T, Cond> = T extends Cond ? [] : [T];

export type Expand<T> = T extends infer O ? { [K in keyof O]: O[K] } : never;

// I think TS will only map over a union type if you use a conditional - @brendonovich
export type ProcedureKeyTuple<P extends ProcedureDef> = P extends ProcedureDef
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

export interface ClientArgs<TProcs extends ProceduresDef>
  extends TRPCClientArgs {
  filter?: FilterFn<TProcs>;
}

function clientFactory<
  TProcs extends ProceduresDef,
  TArgs extends ClientArgs<TProcs>
>(args: TArgs) {
  type FilteredProcs = ApplyFilter<TProcs, TArgs>;

  type Queries = FilteredProcs["queries"];
  type Query<K extends Queries["key"]> = GetProcedure<Queries, K>;
  type Mutations = FilteredProcs["mutations"];
  type Mutation<K extends Mutations["key"]> = GetProcedure<Mutations, K>;
  type Subscriptions = FilteredProcs["subscriptions"];
  type Subscription<K extends Subscriptions["key"]> = GetProcedure<
    Subscriptions,
    K
  >;

  const runtime = {
    transformer: getTransformer(args),
    onError: args.onError,
  };

  let links: OperationLink<any>[];
  let requestId = 0;

  function setLinks(newLink: TRPCLink<any>[]) {
    links = newLink.map((link) => link(runtime));
  }

  if (args.links) setLinks(args.links);

  function $request<TProc extends ProcedureDef>(requestArgs: {
    type: Operation["type"];
    procedureKey: ProcedureKeyTuple<TProc>;
    context?: OperationContext;
  }) {
    const { procedureKey } = args.filter
      ? args.filter({
          procedureKey: requestArgs.procedureKey,
        })
      : { procedureKey: requestArgs.procedureKey };

    return createChain<inferProcedures<any>, TProc["input"], TProc["result"]>({
      links: links as OperationLink<any, any, any>[],
      op: {
        id: ++requestId,
        type: requestArgs.type,
        path: procedureKey[0],
        input: procedureKey[1],
        context: requestArgs.context ?? {},
      },
    }).pipe(share());
  }

  function requestAsPromise<TProc extends ProcedureDef>(opts: {
    type: Operation["type"];
    procedureKey: ProcedureKeyTuple<TProc>;
    context?: OperationContext;
    signal?: AbortSignal;
  }) {
    const req$ = $request(opts);
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

  return {
    query<K extends Queries["key"] & string>(
      keyAndInput: ProcedureKeyTuple<Query<K>>,
      opts?: TRPCRequestOptions
    ) {
      return requestAsPromise({
        type: "query",
        procedureKey: keyAndInput,
        context: opts?.context,
        signal: opts?.signal,
      });
    },

    mutation<K extends Mutations["key"] & string>(
      keyAndInput: ProcedureKeyTuple<Mutation<K>>,
      opts?: TRPCRequestOptions
    ) {
      return requestAsPromise({
        type: "mutation",
        procedureKey: keyAndInput,
        context: opts?.context,
        signal: opts?.signal,
      });
    },

    subscription<K extends Subscriptions["key"] & string>(
      keyAndInput: ProcedureKeyTuple<Subscription<K>>,
      opts: TRPCRequestOptions &
        Partial<TRPCSubscriptionObserver<Subscription<K>["result"], RSPCError>>
    ) {
      const observable$ = $request({
        type: "subscription",
        procedureKey: keyAndInput,
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
    },

    setLinks,

    /*
     * @internal
     * */
    __procs: null as unknown as TProcs,
    __args: null as unknown as TArgs,
  };
}

export type Client<
  TProcs extends ProceduresDef,
  TArgs extends ClientArgs<TProcs>
> = Omit<ReturnType<typeof clientFactory<TProcs, TArgs>>, "setLinks">;

export type ClientFilteredProcs<C extends Client<any, any>> = ApplyFilter<
  C["__procs"],
  C["__args"]
>;

export function createRspcVanilla<TProcs extends ProceduresDef>() {
  return {
    createClient<TArgs extends ClientArgs<TProcs>>(args: TArgs) {
      type Client = ReturnType<typeof clientFactory<TProcs, TArgs>>;

      return clientFactory<TProcs, TArgs>(args) as unknown as Expand<
        TArgs extends {
          links: any;
        }
          ? Omit<Client, "setLinks">
          : Client
      >;
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

const rspc = createRspcVanilla<Procedures>();

const regularClient = rspc.createClient({
  links: [],
});
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
// const apiHooks = createR
