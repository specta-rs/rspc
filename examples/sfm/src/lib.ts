const batch: any[] = [];

export async function doMutation(op: string) {
  batch.push(["MUTATION", op]);
  await new Promise((resolve) => setTimeout(resolve, 1000));
}

export async function doQuery(op: string) {
  batch.push(["query", op]);
  await new Promise((resolve) => setTimeout(resolve, 1000));
}

export function printBatch() {
  console.log(batch);
}
