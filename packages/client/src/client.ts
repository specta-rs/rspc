// TODO: Redo this entire system when links are introduced
import {
  RSPCError,
  ProceduresLike,
  inferQueryResult,
  ProceduresDef,
  inferMutationResult,
  inferProcedures,
  inferSubscriptionResult,
  _inferInfiniteQueryProcedureHandlerInput,
  _inferProcedureHandlerInput,
} from ".";
import { randomId, Transport } from "./transport";

// TODO
export interface SubscriptionOptions<TOutput> {
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

// TODO
export interface ClientArgs {
  transport: Transport;
  onError?: (err: RSPCError) => void | Promise<void>;
}

// TODO
export function createClient<TProcedures extends ProceduresLike>(
  args: ClientArgs
): Client<inferProcedures<TProcedures>> {
  return new Client(args);
}

// TODO
export class Client<TProcedures extends ProceduresDef> {
  public _rspc_def: ProceduresDef = undefined!;
  private transport: Transport;
  private subscriptionMap = new Map<string, (data: any) => void>();
  private onError?: (err: RSPCError) => void | Promise<void>;

  constructor(args: ClientArgs) {
    this.transport = args.transport;
    this.transport.clientSubscriptionCallback = (id, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
    this.onError = args.onError;
  }

  async query<K extends TProcedures["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ]
  ): Promise<inferQueryResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest(
        "query",
        keyAndInput[0],
        keyAndInput[1]
      );
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }
      throw err;
    }
  }

  async mutation<K extends TProcedures["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "mutations", K>
    ]
  ): Promise<inferMutationResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest(
        "mutation",
        keyAndInput[0],
        keyAndInput[1]
      );
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }
      throw err;
    }
  }

  // TODO: Redesign this, i'm sure it probably has race conditions but it works for now
  addSubscription<
    K extends TProcedures["subscriptions"]["key"] & string,
    TData = inferSubscriptionResult<TProcedures, K>
  >(
    keyAndInput: [
      K,
      _inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: SubscriptionOptions<TData>
  ): () => void {
    try {
      let subscriptionId = randomId();
      let unsubscribed = false;

      const cleanup = () => {
        this.subscriptionMap?.delete(subscriptionId);
        if (subscriptionId) {
          this.transport.doRequest(
            "subscriptionStop",
            undefined!,
            subscriptionId
          );
        }
      };

      this.transport.doRequest("subscription", keyAndInput[0], [
        subscriptionId,
        keyAndInput[1],
      ]);

      if (opts.onStarted) opts.onStarted();
      this.subscriptionMap?.set(subscriptionId, opts.onData);

      return () => {
        unsubscribed = true;
        cleanup();
      };
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      }

      return () => {};
    }
  }
}
