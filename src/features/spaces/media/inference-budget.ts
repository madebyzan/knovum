import pLimit from "p-limit";

const inference = pLimit(1);

export const runSpaceMediaInference = <T>(operation: () => Promise<T>) => inference(operation);
