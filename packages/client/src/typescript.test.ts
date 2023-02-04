import { describe, it } from "vitest";
import { inferProcedureInput, inferProcedureResult, inferQueryInput } from ".";
import { assertTy } from "./utils.test";

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
    assertTy<inferProcedureInput<Procedures, "queries", "noArgQuery">, never>();
    assertTy<
      inferProcedureResult<Procedures, "queries", "noArgQuery">,
      string
    >();

    assertTy<
      inferProcedureInput<Procedures, "queries", "singleArgQuery">,
      number
    >();
    assertTy<
      inferProcedureResult<Procedures, "queries", "singleArgQuery">,
      number
    >();
  });

  it("mutation inference helpers", async () => {
    assertTy<
      inferProcedureInput<Procedures, "mutations", "noArgMutation">,
      never
    >();
    assertTy<
      inferProcedureResult<Procedures, "mutations", "noArgMutation">,
      string
    >();

    assertTy<
      inferProcedureInput<Procedures, "mutations", "singleArgMutation">,
      number
    >();
    assertTy<
      inferProcedureResult<Procedures, "mutations", "singleArgMutation">,
      number
    >();
  });

  it("subscription inference helpers", async () => {
    assertTy<
      inferProcedureInput<Procedures, "subscriptions", "noArgSubscription">,
      never
    >();
    assertTy<
      inferProcedureResult<Procedures, "subscriptions", "noArgSubscription">,
      string
    >();

    assertTy<
      inferProcedureInput<Procedures, "subscriptions", "singleArgSubscription">,
      boolean
    >();
    assertTy<
      inferProcedureResult<
        Procedures,
        "subscriptions",
        "singleArgSubscription"
      >,
      boolean
    >();
  });
});
