import { describe, expect, it } from "vitest";

import { runSpaceAssetProcessing, runSpaceAssetTransfer } from "../concurrency-budget";

async function exerciseBudget(submit: <T>(operation: () => Promise<T>) => Promise<T>, limit: number) {
  const releases: Array<() => void> = [];
  let active = 0;
  let peak = 0;
  const work = Array.from({ length: limit + 1 }, () =>
    submit(async () => {
      active += 1;
      peak = Math.max(peak, active);
      await new Promise<void>((resolve) => releases.push(resolve));
      active -= 1;
    }),
  );
  await Promise.resolve();
  await Promise.resolve();
  expect(active).toBe(limit);
  const first = releases.shift();
  if (!first) throw new Error("Expected a submitted operation");
  first();
  await work[0];
  await Promise.resolve();
  expect(peak).toBe(limit);
  for (const release of releases) release();
  await Promise.all(work);
  expect(active).toBe(0);
  return peak;
}

describe("capability submission budgets", () => {
  it("keeps transfer and processing at two concurrent operations", async () => {
    expect(await exerciseBudget(runSpaceAssetTransfer, 2)).toBe(2);
    expect(await exerciseBudget(runSpaceAssetProcessing, 2)).toBe(2);
  });
});
