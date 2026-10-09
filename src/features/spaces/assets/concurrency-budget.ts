import pLimit from "p-limit";

const transfers = pLimit(2);
const processing = pLimit(2);

export function runSpaceAssetTransfer<T>(operation: () => Promise<T>) {
  return transfers(operation);
}
export function runSpaceAssetProcessing<T>(operation: () => Promise<T>) {
  return processing(operation);
}
