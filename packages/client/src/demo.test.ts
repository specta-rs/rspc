import { describe, it, expect } from "vitest";

function expectType<ExpectedType>(_value: ExpectedType) {}

function todo_temp() {
  return "Hello World";
}

describe("demo suite", () => {
  it("test assert", async () => {
    const result = "123";
    expect(result).toMatch("123");
  });

  it("test types", async () => {
    expectType<string>(todo_temp());
  });
});
