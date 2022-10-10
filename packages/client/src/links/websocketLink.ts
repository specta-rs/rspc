import { RSPCLink } from "..";

export const websocketLink: RSPCLink = (url: string) => {
  const ws = new WebSocket(url);
  // this.attachEventListeners();

  return () => {
    return {
      next(value) {},
      error(value) {},
      complete(value) {},
    };
  };
};
