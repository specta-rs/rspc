import { describe, it } from "vitest";

export function assertTy<T>(_: T);
export function assertTy<T extends U, U>();
export function assertTy() {}

export type FunctionLike = (...args: any[]) => any;
export type HasProperty<
  T extends { [x: string]: any },
  Prop extends string
> = Prop extends keyof T ? true : false;

describe("Test utils", () => {
  it("assert", async () => {
    assertTy<true, true>();
    // @ts-expect-error
    assertTy<true, false>();

    assertTy<true>(true);
    // @ts-expect-error
    assertTy<true>(false);
  });

  it("FunctionLike", async () => {
    assertTy<() => string, FunctionLike>();
    assertTy<() => void, FunctionLike>();
    assertTy<(x: number) => void, FunctionLike>();
    assertTy<(x: number, y: string, z: boolean) => void, FunctionLike>();
  });

  it("HasProperty", async () => {
    const obj = { demo: "hello" };
    assertTy<HasProperty<typeof obj, "demo">, true>();
    assertTy<HasProperty<typeof obj, "demo2">, false>();
  });
});
