import { observable, RSPCLink } from "..";

export const loggerLink: RSPCLink = (url: string) => {
  return ({ next }) => {
    return observable(() => {
      return next.subscribe({
        next(value) {
          console.log("VALUE:", value);
        },
        error(err) {
          console.log("ERR:", err);
        },
        complete() {},
      });
    });
  };
};
