// TODO: This entire folder provides an interop layer between the old and new syntax because the new syntax isn't ready for prime time. The whole folder is tech debt and will be removed in time.

import {
  RSPCError,
  ProceduresDef,
  inferProcedure,
  inferSubscriptionResult,
  inferMutationResult,
  inferQueryResult,
} from "..";
export * from "./error";
export * from "./typescript";

export class FetchTransport {
  constructor(url: string) {
    // TODO
  }
}

/**
 * @deprecated
 */
export type ProceduresLike =
  | {
      _rspc_def: ProceduresDef;
    }
  | ProceduresDef;

/**
 * @deprecated
 */
export type inferProcedures<TProcedures extends ProceduresLike> =
  TProcedures extends {
    _rspc_def: ProceduresDef;
  }
    ? TProcedures["_rspc_def"]
    : TProcedures;

/**
 * @deprecated This helper will be made internal on the next release so don't rely on it!
 */
export type _inferProcedureHandlerInput<
  TProcedures extends ProceduresLike,
  TOperation extends keyof ProceduresDef,
  K extends inferProcedures<TProcedures>[TOperation]["key"]
> = inferProcedure<TProcedures, TOperation, K>["input"] extends never
  ? []
  : [inferProcedure<TProcedures, TOperation, K>["input"]];

export interface Transport {
  // TODO
  // clientSubscriptionCallback?: (id: string, key: string, value: any) => void;
  // doRequest(operation: OperationType, key: string, input: any): Promise<any>;
}

interface ClientArgs {
  transport: Transport;
  onError?: (err: RSPCError) => void | Promise<void>;
}

export function createClient<TProcedures extends ProceduresLike>(
  args: ClientArgs
): Client<inferProcedures<TProcedures>> {
  return new Client(args);
}

/**
 * @deprecated Will be removed when interop API is removed!
 */
export interface SubscriptionOptions<TOutput> {
  onStarted?: () => void;
  onData: (data: TOutput) => void;
  onError?: (err: RSPCError) => void;
}

export class Client<TProcedures extends ProceduresDef> {
  _rspc_def: ProceduresDef;
  private onError?: (err: RSPCError) => void | Promise<void>;

  constructor(args: ClientArgs) {
    this._rspc_def = undefined as unknown as TProcedures;
    this.onError = args.onError;

    // TODO
  }

  query<K extends TProcedures["queries"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "queries", K>
    ]
  ): Promise<inferQueryResult<TProcedures, K>> {
    return undefined as any; // TODO
  }

  mutation<K extends TProcedures["mutations"]["key"] & string>(
    keyAndInput: [
      key: K,
      ...input: _inferProcedureHandlerInput<TProcedures, "mutations", K>
    ]
  ): Promise<inferMutationResult<TProcedures, K>> {
    return undefined as any; // TODO
  }

  addSubscription<K extends TProcedures["subscriptions"]["key"] & string>(
    keyAndInput: [
      K,
      ..._inferProcedureHandlerInput<TProcedures, "subscriptions", K>
    ],
    opts: SubscriptionOptions<inferSubscriptionResult<TProcedures, K>>
  ): () => void {
    return undefined as any; // TODO
  }
}
