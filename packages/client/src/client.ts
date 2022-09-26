// TODO: Redo this entire system when links are introduced
import {
  RSPCError,
  ProceduresLike,
  inferQueryResult,
  ProceduresDef,
  inferQueryInput,
  inferMutationInput,
  inferMutationResult,
  inferSubscriptionInput,
  inferProcedures,
  inferSubscriptionResult,
} from ".";
import { Transport } from "./transport";

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
    this.transport.clientSubscriptionCallback = (id, key, value) => {
      const func = this.subscriptionMap?.get(id);
      if (func !== undefined) func(value);
    };
    this.subscriptionMap = new Map();
    this.onError = args.onError;
  }

  async query<K extends TProcedures["queries"]["key"] & string>(
    key: K,
    input: inferQueryInput<TProcedures, K>
  ): Promise<inferQueryResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest("query", key, input);
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      } else {
        throw err;
      }
    }
  }

  async mutation<K extends TProcedures["mutations"]["key"] & string>(
    key: K,
    input: inferMutationInput<TProcedures, K>
  ): Promise<inferMutationResult<TProcedures, K>> {
    try {
      return await this.transport.doRequest("mutation", key, input);
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      } else {
        throw err;
      }
    }
  }

  // TODO: Redesign this, i'm sure it probably has race conditions but it works for now
  addSubscription<
    K extends TProcedures["mutations"]["key"] & string,
    TData = inferSubscriptionResult<TProcedures, K>
  >(
    key: K,
    input: inferSubscriptionInput<TProcedures, K>,
    opts: SubscriptionOptions<TData>
  ): () => void {
    try {
      let subscriptionId: string = undefined!;
      let unsubscribed = false;

      const cleanup = () => {
        this.subscriptionMap?.delete(subscriptionId);
        if (subscriptionId) {
          this.transport.doRequest(
            "subscriptionStop",
            subscriptionId,
            undefined
          );
        }
      };

      this.transport.doRequest("subscription", key, input).then((id) => {
        subscriptionId = id;
        if (unsubscribed) {
          cleanup();
        } else {
          if (opts.onStarted) opts.onStarted();
          this.subscriptionMap?.set(subscriptionId, opts.onData);
        }
      });

      return () => {
        unsubscribed = true;
        cleanup();
      };
    } catch (err) {
      if (this.onError) {
        this.onError(err as RSPCError);
      } else {
        throw err;
      }

      return () => {};
    }
  }
}
