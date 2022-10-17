// https://stackoverflow.com/a/54487392
export type OmitDistributive<T, K extends PropertyKey> = T extends any
  ? T extends object
    ? Id<OmitRecursively<T, K>>
    : T
  : never;
export type Id<T> = {} & { [P in keyof T]: T[P] }; // Cosmetic use only makes the tooltips expand the type can be removed
export type OmitRecursively<T extends any, K extends PropertyKey> = Omit<
  { [P in keyof T]: OmitDistributive<T[P], K> },
  K
>;
