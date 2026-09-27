import pLimit from "p-limit";

const transfers = pLimit(2);
const processing = pLimit(2);

export const runSpaceAssetTransfer = <T>(operation: () => Promise<T>) => transfers(operation);
export const runSpaceAssetProcessing = <T>(operation: () => Promise<T>) => processing(operation);
