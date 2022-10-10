// import { RSPCLink } from "..";

import Observable from "zen-observable";

// export interface HttpLinkOptions {
//   url: string;
//   headers?: Record<string, string> | (() => Record<string, string>);
// }

export const fetchObservable = new Observable((observer) => {
  const ac = new AbortController();

  const req = fetch("https://jsonplaceholder.typicode.com/posts", {
    signal: ac.signal,
  })
    .then((res) => {
      if (res.ok) {
        return res.json();
      } else {
        observer.error(new Error("TODO"));
      }
    })
    .then((body) => {
      observer.next(body);
      observer.complete();
    });
});

// export const httpLink: RSPCLink = ({ url }: HttpLinkOptions) => {
//   return ({ op }) => {
//     // const { path, input, type } = op;
//     // const { promise, cancel } = httpRequest({
//     //   url,
//     //   runtime,
//     //   type,
//     //   path,
//     //   input,
//     // });
//     // promise
//     //   .then((res) => {
//     //   //   const transformed = transformResult(res.json, runtime);
//     //   //   if (!transformed.ok) {
//     //   //     observer.error(
//     //   //       TRPCClientError.from(transformed.error, {
//     //   //         meta: res.meta,
//     //   //       })
//     //   //     );
//     //   //     return;
//     //   //   }
//     //   //   observer.next({
//     //   //     context: res.meta,
//     //   //     result: transformed.result,
//     //   //   });
//     //   //   observer.complete();
//     //   // })
//     //   // .catch((cause) => observer.error(TRPCClientError.from(cause)));
//     // return () => {
//     //   cancel();
//     // };
//   };
// };

export {};
