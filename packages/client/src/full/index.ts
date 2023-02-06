import { FullObservable } from "..";
import { Observable, observableToPromise } from "../internals/observable";

export * from "./wsLink";

export * from "../internals/observable";

// TODO: Move this somewhere else
// TODO: Remove `any`
export function fullObservable(ob$: Observable<any, any>): FullObservable {
  return {
    exec: () => {
      const { promise, abort } = observableToPromise(ob$);
      return { promise, cancel: abort };
    },
    subscribe(opts) {
      throw new Error("TODO: Sub method");
    },
  };
}

// const exec = (opts: InitRspcInnerArgs, initialOp: LinkOperation) => {
//   const { promise, abort } = observable((observer) => {
//     function execute(index = 0, op = initialOp) {
//       const next = opts.links[index];
//       if (!next) {
//         throw new Error(
//           "No more links to execute - did you forget to add a terminating link?"
//         );
//       }

//       const subscription = next({
//         op: op.op,
//         next(nextOp) {
//           const nextObserver = execute(index + 1, nextOp);

//           return nextObserver;
//         },
//       });
//       return subscription;
//     }

//     const obs$ = execute();
//     if (isFullObservable(obs$)) {
//       obs$.subscribe({
//         next: (v) => {
//           observer.next(v);
//         },
//         error: (e) => {
//           observer.error(e);
//         },
//         complete: () => {
//           observer.complete();
//         },
//       });
//     } else {
//       // TODO: Do the thing
//       throw new Error("TODO: Convert into observable!");
//     }
//   });

//   return { promise, cancel: abort };
// };

// BREAK

// if (isFakeObservable(resp)) {
// } else {
//   throw new Error("TODO");
// }

// const observable = exec(opts, {
//   op: {
//     type: "query",
//     path: keyAndInput[0] as any,
//     input: keyAndInput[1] as any,
//     context: {},
//   },
//   next() {
//     throw new Error("TODO: Probally unreachable"); // TODO: Deal with this
//   },
// });

// const { promise, abort } = observableToPromise(observable);
// TODO: Expose `abort` function to user if they want it -> Maybe an arg, idk how tRPC do it?

// TODO: Should we expose `v.context`???
// // @ts-expect-error // TODO: Fix type error at some point
// return promise.then((v) => v.result.data);
