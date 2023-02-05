export function getWindow() {
  if (typeof window !== "undefined") {
    return window;
  }
  return globalThis;
}

export function getAbortController(
  ac: typeof AbortController | undefined | null
): typeof AbortController | null {
  return ac ?? getWindow().AbortController ?? null;
}
