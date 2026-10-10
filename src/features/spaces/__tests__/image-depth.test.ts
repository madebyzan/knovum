import { beforeEach, expect, it, vi } from "vitest";

import { defaultImageEdit } from "../image-editing/image-edit";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { imageSourceNode } from "./image-operation-fixtures";
const calls = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args: unknown) => Promise<void>>(),
  save: vi.fn<(options: unknown) => Promise<string | null>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: calls.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ save: calls.save }));
import { exportSpaceNode } from "../assets/node-export";

function depthNode() {
  const node = imageSourceNode();
  return {
    ...node,
    data: {
      ...node.data,
      imageDepth: {
        encoding: "relative-inverse-depth-u16-v1" as const,
        modelSpecId: "depth-spec",
      },
    },
  };
}
beforeEach(() => {
  calls.invoke.mockReset().mockResolvedValue(undefined);
  calls.save.mockReset().mockResolvedValue("/exports/depth.png");
});
it("exports the original depth asset without sending it through the photo renderer", async () => {
  const node = depthNode();
  await expect(exportSpaceNode("space", node)).resolves.toBe(true);
  expect(calls.invoke).toHaveBeenCalledWith("plugin:spaces|export_asset", {
    sourcePath: node.data.assetPath,
    destinationPath: "/exports/depth.png",
  });
  expect(calls.invoke).toHaveBeenCalledOnce();
});
it("rejects photo adjustments on a depth analysis field", () => {
  expect(() => validateSpaceNodePayload(depthNode().data)).not.toThrow();
  expect(() => validateSpaceNodePayload({ ...depthNode().data, imageEdit: defaultImageEdit() })).toThrow(
    "Relative depth fields do not support photo edits",
  );
});
