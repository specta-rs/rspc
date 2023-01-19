import { ProcedureDef } from "@rspc/client";
import { ProceduresDef } from "@rspc/client";
import { NormiOptions } from ".";
import { OmitRecursively } from "./utils";

// export type NormiClient<TProcedures extends ProceduresDef> = Client<
//   TProcedures,
//   Normalized<TProcedures["queries"]>,
//   Normalized<TProcedures["mutations"]>,
//   Normalized<TProcedures["subscriptions"]>
// >;

/**
 * is responsible for normalizing the Typescript type before the type is exposed back to the user.
 *
 * @internal
 */
export type Normalized<T extends ProcedureDef> = T extends any
  ? {
      key: T["key"];
      // TODO: Typescript transformation for arrays
      result: OmitRecursively<T["result"], "$id" | "$type">;
      input: T["input"];
    }
  : never;

// export function normiHooks<TProcedures extends ProceduresDef>(
//   opts: NormiOptions
// ): CustomHooks<
//   TProcedures,
//   // TODO: Remove this exclude for prod. It's just for testing.
//   Exclude<Normalized<TProcedures["queries"]>, { key: "version" }>,
//   Normalized<TProcedures["mutations"]>,
//   Normalized<TProcedures["subscriptions"]>
// > {
//   return {
//     _def: undefined as any,
//     serialize(data: any) {
//       console.log("IN", data);

//       return data;
//     },
//     deserialize(data: any) {
//       console.log("OUT", data);

//       return data;
//     },
//     // async useQuery() {
//     //   console.log("Bruh");
//     // },
//   };
// }

// export function createNormiHooks<
//   TBaseProcedures extends ProceduresDef,
//   TContext,
//   TQueries extends ProcedureDef = TBaseProcedures["queries"],
//   TMutations extends ProcedureDef = TBaseProcedures["mutations"],
//   TSubscriptions extends ProcedureDef = TBaseProcedures["subscriptions"]
// >(
//   hookCreateFn: HookCreateFunction<
//     TContext,
//     TBaseProcedures,
//     TQueries,
//     TMutations,
//     TSubscriptions
//   >,
//   opts: NormiOptions
// ): Hooks<
//   TContext,
//   TBaseProcedures,
//   Exclude<Normalized<TBaseProcedures["queries"]>, { key: "version" }>,
//   Normalized<TBaseProcedures["mutations"]>,
//   Normalized<TBaseProcedures["subscriptions"]>
// > {
//   return hookCreateFn({
//     internal: {
//       customHooks: () => ({
//         useQuery: async (keyAndInput, next) => {
//           console.log("IN", keyAndInput);

//           const result = await next(keyAndInput);

//           console.log("OUT", result);

//           return result;
//         },
//       }),
//     },
//   });
// }
