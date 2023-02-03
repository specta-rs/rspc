import { ClientArgs } from ".";

/**
 * @public
 */
export type DataTransformer = {
  serialize(object: any): any;
  deserialize(object: any): any;
};

/**
 * @public
 */
export type CombinedDataTransformer = {
  input: DataTransformer;
  output: DataTransformer;
};

/**
 * @public
 */
export type CombinedDataTransformerClient = {
  input: Pick<DataTransformer, "serialize">;
  output: Pick<DataTransformer, "deserialize">;
};

export type ClientDataTransformerOptions =
  | DataTransformer
  | CombinedDataTransformerClient;

export function getTransformer(opts: ClientArgs<any>): DataTransformer {
  if (!opts.transformer)
    return {
      serialize: (data) => data,
      deserialize: (data) => data,
    };
  if ("input" in opts.transformer)
    return {
      serialize: opts.transformer.input.serialize,
      deserialize: opts.transformer.output.deserialize,
    };
  return opts.transformer;
}
