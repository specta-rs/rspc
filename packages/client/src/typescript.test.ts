import { describe, it } from "vitest";
import { inferProcedureInput, inferProcedureResult, inferQueryInput } from ".";

export function assertTypeEquality<T extends never>() {}
type Equals<A, B> = Exclude<A, B> | Exclude<B, A>;

export type Procedures = {
  queries:
    | { key: "flatteningQuery"; input: Flatten; result: Flatten }
    | { key: "noArgQuery"; input: never; result: string }
    | {
        key: "paginatedQueryCursorAndArg";
        input: PaginatedQueryArg2;
        result: MyPaginatedData;
      }
    | {
        key: "paginatedQueryOnlyCursor";
        input: PaginatedQueryArg;
        result: MyPaginatedData;
      }
    | { key: "singleArgQuery"; input: number; result: number };
  mutations:
    | { key: "noArgMutation"; input: never; result: string }
    | { key: "singleArgMutation"; input: number; result: number };
  subscriptions:
    | { key: "noArgSubscription"; input: never; result: string }
    | { key: "singleArgSubscription"; input: boolean; result: boolean };
};

export type Flatten = { Unnamed: string };

export type MyPaginatedData = { data: string[]; next_cursor: string | null };

export type PaginatedQueryArg = { cursor: string };

export type PaginatedQueryArg2 = { cursor: string; my_param: number };

export type Test = "Unit" | { Unnamed: string };

describe("Typescript", () => {
  it("query inference helpers", async () => {
    assertTypeEquality<
      Equals<never, inferProcedureInput<Procedures, "queries", "noArgQuery">>
    >();
    assertTypeEquality<
      Equals<string, inferProcedureResult<Procedures, "queries", "noArgQuery">>
    >();

    assertTypeEquality<
      Equals<
        number,
        inferProcedureInput<Procedures, "queries", "singleArgQuery">
      >
    >();
    assertTypeEquality<
      Equals<
        number,
        inferProcedureResult<Procedures, "queries", "singleArgQuery">
      >
    >();
  });

  it("mutation inference helpers", async () => {
    assertTypeEquality<
      Equals<
        never,
        inferProcedureInput<Procedures, "mutations", "noArgMutation">
      >
    >();
    assertTypeEquality<
      Equals<
        string,
        inferProcedureResult<Procedures, "mutations", "noArgMutation">
      >
    >();

    assertTypeEquality<
      Equals<
        number,
        inferProcedureInput<Procedures, "mutations", "singleArgMutation">
      >
    >();
    assertTypeEquality<
      Equals<
        number,
        inferProcedureResult<Procedures, "mutations", "singleArgMutation">
      >
    >();
  });

  it("subscription inference helpers", async () => {
    assertTypeEquality<
      Equals<
        never,
        inferProcedureInput<Procedures, "subscriptions", "noArgSubscription">
      >
    >();
    assertTypeEquality<
      Equals<
        string,
        inferProcedureResult<Procedures, "subscriptions", "noArgSubscription">
      >
    >();

    assertTypeEquality<
      Equals<
        boolean,
        inferProcedureInput<
          Procedures,
          "subscriptions",
          "singleArgSubscription"
        >
      >
    >();
    assertTypeEquality<
      Equals<
        boolean,
        inferProcedureResult<
          Procedures,
          "subscriptions",
          "singleArgSubscription"
        >
      >
    >();
  });
});
