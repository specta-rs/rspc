import {
  OperationContext,
  OperationLink,
  Operation,
  share,
  inferObservableValue,
  observableToPromise,
  RSPCError,
  TRPCLink,
  OnErrorFunction,
} from "..";
import { createChain } from "../links/internals/createChain";
import { ProcedureDef, ProceduresDef } from "../bindings";
import { Expand, GetProcedure, ProcedureKeyTuple } from "./utils";
import { ClientDataTransformerOptions, getTransformer } from "./transformer";
import { ApplyFilter, FilterFn } from "./filters";

export * from "./utils";
export * from "./filters";
export * from "./transformer";

export interface ClientArgs<TProcs extends ProceduresDef> {
  /**
   * Data transformer
   * @link https://trpc.io/docs/data-transformers
   **/
  transformer?: ClientDataTransformerOptions;
  /**
   * TODO
   * @link https://rspc.dev/todo
   **/
  links?: TRPCLink<any>[];
  /**
   * TODO
   * @link https://rspc.dev/todo
   **/
  onError?: OnErrorFunction;

  filter?: FilterFn<TProcs>;
}

export interface SubscriptionObserver<TValue, TError> {
  onStarted: () => void;
  onData: (value: TValue) => void;
  onError: (err: TError) => void;
  onStopped: () => void;
  onComplete: () => void;
}

export interface TRPCRequestOptions {
  /**
   * Pass additional context to links
   */
  context?: OperationContext;
  signal?: AbortSignal;
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

    return createChain<FilteredProcs, TProc["input"], TProc["result"]>({
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
        Partial<SubscriptionObserver<Subscription<K>["result"], RSPCError>>
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

export function createRspcRoot<TProcs extends ProceduresDef>() {
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
