export type Maybe<TType> = TType | undefined | null;

export function getWindow() {
  if (typeof window !== "undefined") {
    return window;
  }
  return globalThis;
}
export function getAbortController(
  ac: Maybe<typeof AbortController>
): typeof AbortController | null {
  return ac ?? getWindow().AbortController ?? null;
}

export function getFetch(f?: typeof fetch): typeof fetch {
  if (f) {
    return f;
  }

  const win = getWindow();
  const globalFetch = win.fetch;
  if (globalFetch) {
    return typeof globalFetch.bind === "function"
      ? globalFetch.bind(win)
      : globalFetch;
  }

  throw new Error("No fetch implementation found");
}
