import { expect, it } from "vitest";

import { stepTrimFrameMs } from "../clip-bounds";
import { createVideoEditProject } from "../project";
import { stepOutputFrame } from "../timing";
it("steps exactly thirty output frames in one second without accumulating millisecond drift", () => {
  const project = createVideoEditProject({
    nodeId: "source",
    sourceRevision: "revision",
    durationMs: 2000,
    sourceWidth: 1280,
    sourceHeight: 720,
  });
  let position = 0;
  for (let frame = 0; frame < 30; frame++) position = stepOutputFrame(project, position, 1);
  expect(position).toBeCloseTo(1000, 8);
  expect(stepOutputFrame(project, 0, -1)).toBe(0);
  expect(stepOutputFrame(project, 2000, 1)).toBe(2000);
});

it("keeps integer persisted trim edges on the frame grid through repeated keyboard steps", () => {
  let position = 0;
  for (let frame = 0; frame < 30; frame++) position = stepTrimFrameMs(position, 30, 1);
  expect(position).toBe(1000);
  for (let frame = 0; frame < 30; frame++) position = stepTrimFrameMs(position, 30, -1);
  expect(position).toBe(0);
});
