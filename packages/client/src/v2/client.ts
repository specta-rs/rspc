import type {
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
  inferProcedureResult,
} from "..";
import {
  AlphaRSPCError,
  Link,
  OperationContext,
  Operation,
  LinkResult,
} from "../v2";

// TODO
export interface SubscriptionOptions<TOutput> {
  // onStarted?: () => void;
  onData: (data: TOutput) => void;
  // TODO: Probs remove `| Error` here
  onError?: (err: AlphaRSPCError | Error) => void;
}

type KeyAndInput = [string] | [string, any];

type OperationOpts = {
  signal?: AbortSignal;
  context?: OperationContext;
  // skipBatch?: boolean; // TODO: Make this work + add this to React
};

// TODO
type ClientArgs<TSubClients extends Record<string, AlphaClient<any>> = {}> = {
  onError?: (err: AlphaRSPCError) => void | Promise<void>;
} & (
  | {
      links: Link[];
      subClients?: TSubClients;
    }
  | {
      dangerously_mapQueryKey?: MapQueryKeyFn;
    }
);

export function initRspc<P extends ProceduresDef>(args: ClientArgs) {
  return new AlphaClient<P>(args);
}

const generateRandomId = () => Math.random().toString(36).slice(2);

// TODO: This will replace old client
export class AlphaClient<
  P extends ProceduresDef,
  TSubClients extends Record<string, AlphaClient<any>> = {}
> {
  public _rspc_def: ProceduresDef = undefined!;
  private onError?: (err: AlphaRSPCError) => void | Promise<void>;
  /** @internal */
  public _subClients_def: TSubClients = null as any;
  /** @internal */
  _procedures_def: P = null as any;

  private state:
    | { subClients: TSubClients; links: Link[] }
    | { parent: AlphaClient<any>; dangerously_mapQueryKey?: MapQueryKeyFn };

  constructor(args: ClientArgs<TSubClients>) {
    this.onError = args.onError;

    if ("links" in args) {
      if (args.links.length === 0) {
        throw new Error("Must provide at least one link");
      }

      this.state = {
        links: args.links,
        subClients: args.subClients ?? ({} as any),
      };

      Object.values(args.subClients ?? {}).forEach((client) =>
        client.setParent(this)
      );
    } else {
      this.state = {
        parent: null!,
        dangerously_mapQueryKey: args.dangerously_mapQueryKey,
      };
    }
  }

  _mapKeyAndInput(keyAndInput: KeyAndInput) {
    if ("dangerously_mapQueryKey" in this.state)
      return this.state.dangerously_mapQueryKey
        ? this.state.dangerously_mapQueryKey(keyAndInput)
        : keyAndInput;
    else return keyAndInput;
  }

  private exec(op: Operation) {
    const links: Link[] =
      "links" in this.state
        ? this.state.links
        : (this.state.parent as any).links;

    let prevLinkResult: LinkResult = {
      exec: () => {
        throw new Error(
          "rspc: no terminating link was attached! Did you forget to add a 'httpLink' or 'wsLink' link?"
        );
      },
      abort: () => {},
    };

    for (var linkIndex = 0; linkIndex < links.length; linkIndex++) {
      const link = links[links.length - linkIndex - 1]!;
      const result = link({
        op,
        next: () => prevLinkResult,
      });
      prevLinkResult = result;
    }

    return prevLinkResult;
  }

  async query<K extends P["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "queries", K>
    ],
    opts?: OperationOpts
  ): Promise<inferQueryResult<P, K>> {
    try {
      const [path, input] = this._mapKeyAndInput(keyAndInput as any);

      const result = this.exec({
        id: generateRandomId(),
        type: "query",
        input,
        path,
        context: opts?.context || {},
      });

      opts?.signal?.addEventListener("abort", result.abort);

      return await new Promise(result.exec);
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      throw err;
    }
  }

  async mutation<K extends P["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<P, "mutations", K>
    ],
    opts?: OperationOpts
  ): Promise<inferMutationResult<P, K>> {
    try {
      const [path, input] = this._mapKeyAndInput(keyAndInput as any);

      const result = this.exec({
        id: generateRandomId(),
        type: "mutation",
        input,
        path,
        context: opts?.context || {},
      });
      opts?.signal?.addEventListener("abort", result.abort);

      return await new Promise(result.exec);
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      throw err;
    }
  }

  // TODO: Handle resubscribing if the subscription crashes similar to what Tanstack Query does
  addSubscription<
    K extends P["subscriptions"]["key"] & string,
    TData = inferProcedureResult<P, "subscriptions", K>
  >(
    keyAndInput: [K, ..._inferProcedureHandlerInput<P, "subscriptions", K>],
    opts: SubscriptionOptions<TData> & { context?: OperationContext }
  ): () => void {
    try {
      const [path, input] = this._mapKeyAndInput(keyAndInput as any);

      const result = this.exec({
        id: generateRandomId(),
        type: "subscription",
        input,
        path,
        context: opts?.context || {},
      });

      result.exec(
        (data) => opts?.onData(data),
        (err) => opts?.onError?.(err)
      );
      return result.abort;
    } catch (err) {
      if (this.onError) {
        this.onError(err as AlphaRSPCError);
      }
      return () => {};
    }
  }

  subClient<T extends keyof TSubClients>(name: T): TSubClients[T] {
    if ("parent" in this.state)
      throw new Error("SubClient cannot have SubClients!");

    return this.state.subClients[name];
  }

  /**
   * @internal
   */
  setParent(parent: AlphaClient<any, {}>) {
    if ("subClients" in this.state)
      throw new Error("root client cannot have parent!");
    this.state.parent = parent;
  }
}

export function createRspcRoot<P extends ProceduresDef>() {
  return {
    createClientBuilder() {
      return new AlphaClientBuilder<P>();
    },
    dangerously_createClientBuilder<P extends ProceduresDef>(
      args?: AlphaClientBuilderArgs
    ) {
      return new AlphaClientBuilder<P>(args);
    },
  };
}

export function createRspcClient<P extends ProceduresDef>(
  args: ClientArgs<{}>
) {
  return createRspcRoot<P>().createClientBuilder().build(args);
}

type MapQueryKeyFn = (keyAndInput: KeyAndInput) => KeyAndInput;

interface AlphaClientBuilderArgs {
  dangerously_mapQueryKey?: MapQueryKeyFn;
}

type SubClientBuilders = Record<string, AlphaClientBuilder<any, {}>>;
type SubClientBuildersToClients<T extends SubClientBuilders> = {
  [K in keyof T]: T[K] extends AlphaClientBuilder<infer P, {}>
    ? AlphaClient<P, {}>
    : never;
};

export class AlphaClientBuilder<
  P extends ProceduresDef,
  TSubClients extends SubClientBuilders = {}
> {
  private dangerously_mapQueryKey?: MapQueryKeyFn;
  private subClients: TSubClients = {} as any;
  /** @internal */
  _subClients_def: TSubClients = null as any;
  /** @internal */
  _procedures_def: P = null as any;

  constructor(args?: AlphaClientBuilderArgs) {
    this.dangerously_mapQueryKey = args?.dangerously_mapQueryKey;
  }

  addSubClient<
    TName extends string,
    TClientBuilder extends AlphaClientBuilder<any, {}>
  >(
    name: TName,
    client: TClientBuilder
  ): AlphaClientBuilder<
    P,
    {
      [K in keyof TSubClients]: TSubClients[K];
    } & { [K in TName]: TClientBuilder }
  > {
    (this as any).subClients[name] = client;
    return this as any;
  }

  build(args: ClientArgs<{}>) {
    type Clients = SubClientBuildersToClients<TSubClients>;
    return new AlphaClient<P, { [K in keyof Clients]: Clients[K] }>({
      ...args,
      subClients: Object.entries(this.subClients).reduce(
        (acc, [name, builder]) => ({
          ...acc,
          [name]: new AlphaClient({
            dangerously_mapQueryKey: builder.dangerously_mapQueryKey,
            links: [],
          }),
        }),
        {} as any
      ),
    });
  }
}
