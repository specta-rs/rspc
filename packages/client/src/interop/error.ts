// @ts-no-check: TODO: Bruh

export function getMessageFromUnkownError(
  err: unknown,
  fallback: string
): string {
  if (typeof err === "string") {
    return err;
  }

  if (err instanceof Error && typeof err.message === "string") {
    return err.message;
  }
  return fallback;
}

export function getErrorFromUnknown(cause: unknown): Error {
  if (cause instanceof Error) {
    return cause;
  }
  const message = getMessageFromUnkownError(cause, "Unknown error");
  return new Error(message);
}

export function getTRPCErrorFromUnknown(cause: unknown): RSPCError {
  const error = getErrorFromUnknown(cause);
  // this should ideally be an `instanceof TRPCError` but for some reason that isn't working
  // ref https://github.com/trpc/trpc/issues/331
  if (error.name === "RSPCError") {
    return cause as RSPCError;
  }

  // @ts-expect-error: TODO: Fix this
  const trpcError = new RSPCError({
    code: "INTERNAL_SERVER_ERROR",
    cause: error,
    message: error.message,
  });

  // Inherit stack from error
  trpcError.stack = error.stack;

  return trpcError;
}

export function getCauseFromUnknown(cause: unknown) {
  if (cause instanceof Error) {
    return cause;
  }

  return undefined;
}
export class RSPCError extends Error {
  public readonly cause;
  public readonly shape: any; // TODO: Maybe<inferErrorShape<TRouterOrProcedure>>;
  public readonly data: any; // TODO; Maybe<inferErrorShape<TRouterOrProcedure>["data"]>;
  public readonly meta;

  public readonly code: number = 0; // TODO: Hook this back up

  constructor(
    message: string,
    opts?: {
      result?: any; // TODO: Maybe<inferErrorShape<TRouterOrProcedure>>;
      cause?: Error;
      meta?: Record<string, unknown>;
    }
  ) {
    const cause = opts?.cause;

    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore https://github.com/tc39/proposal-error-cause
    super(message, { cause });

    this.meta = opts?.meta;

    this.cause = cause;
    this.shape = opts?.result?.error;
    this.data = opts?.result?.error.data;
    this.name = "RSPCError";

    Object.setPrototypeOf(this, RSPCError.prototype);
  }

  public static from(
    cause: Error, // TODO:  | TRPCErrorResponse<any>,
    opts: { meta?: Record<string, unknown> } = {}
  ): RSPCError {
    if (!(cause instanceof Error)) {
      // @ts-expect-error: TODO
      return new RSPCError(cause.error.message ?? "", {
        ...opts,
        cause: undefined,
        result: cause as any,
      });
    }
    if (cause.name === "TRPCClientError") {
      // @ts-expect-error: TODO: Bruh
      return cause as TRPCClientError<any>;
    }

    return new RSPCError(cause.message, {
      ...opts,
      cause,
      result: null,
    });
  }
}
