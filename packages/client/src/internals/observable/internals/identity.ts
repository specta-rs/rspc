// TODO: It would be nice if this could be inlined (remove when compiled): https://stackoverflow.com/questions/47617077/does-typescript-compiler-has-inline-function-option
export function identity<TType>(x: TType): TType {
  return x;
}
