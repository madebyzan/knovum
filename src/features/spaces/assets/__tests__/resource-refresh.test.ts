import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceFilePayload, SpaceNodePayload } from "../../model/nodes/node-payload";

const { refreshSpaceAsset } = vi.hoisted(() => ({
  refreshSpaceAsset: vi.fn<(spaceId: string, nodeId: string) => Promise<SpaceNodePayload | undefined>>(),
}));

vi.mock("../asset-operations", () => ({ refreshSpaceAsset }));

import { refreshSpaceResources } from "../resource-refresh";

const apply = vi.fn<(spaceId: string, id: string, data: SpaceNodePayload) => void>();

function officeNode(data?: Partial<SpaceFilePayload>): SpaceCanvasNode {
  return {
    id: "office-1",
    type: "file",
    initialPosition: { x: 0, y: 0 },
    data: {
      kind: "file",
      title: "Report",
      assetPath: "/spaces/report.docx",
      fileName: "report.docx",
      size: 42,
      modifiedAt: 1234,
      sourceRevision: "sha256:cached",
      previewText: "cached",
      ...data,
    },
  };
}

describe("refreshSpaceResources", () => {
  beforeEach(() => {
    refreshSpaceAsset.mockReset();
    apply.mockReset();
  });

  it("does not apply node data when refresh reports no change", async () => {
    refreshSpaceAsset.mockResolvedValue(undefined);

    expect(await refreshSpaceResources("space-1", [officeNode()], apply)).toBe(0);
    expect(refreshSpaceAsset).toHaveBeenCalledWith("space-1", "office-1");
    expect(apply).not.toHaveBeenCalled();
  });

  it("applies persisted refresh data when the asset changed", async () => {
    const refreshed: SpaceNodePayload = {
      kind: "file",
      title: "Report",
      assetPath: "/spaces/report.docx",
      fileName: "report.docx",
      size: 48,
      modifiedAt: 5678,
      sourceRevision: "sha256:updated",
      previewText: "updated",
      extractedText: "updated text",
    };
    refreshSpaceAsset.mockResolvedValue(refreshed);

    expect(await refreshSpaceResources("space-1", [officeNode()], apply)).toBe(0);
    expect(apply).toHaveBeenCalledWith("space-1", "office-1", refreshed);
  });

  it("skips nodes in active importing or processing state", async () => {
    const importingNode = { ...officeNode({ processingState: "importing" }), id: "importing-1" };
    const processingNode = { ...officeNode({ processingState: "processing" }), id: "processing-1" };

    expect(await refreshSpaceResources("space-1", [importingNode, processingNode], apply)).toBe(0);
    expect(refreshSpaceAsset).not.toHaveBeenCalled();
    expect(apply).not.toHaveBeenCalled();
  });
});
