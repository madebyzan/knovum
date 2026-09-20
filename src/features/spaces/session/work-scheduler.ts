import pLimit from "p-limit";

const assetTransfers = pLimit(2);
const assetDerivations = pLimit(2);
const mediaInference = pLimit(1);

export const runSpaceAssetTransfer = <T>(operation: () => Promise<T>) => assetTransfers(operation);
export const runSpaceAssetDerivation = <T>(operation: () => Promise<T>) => assetDerivations(operation);
export const runSpaceMediaInference = <T>(operation: () => Promise<T>) => mediaInference(operation);
