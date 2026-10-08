import { vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../../nodes/node-operations";
import type { SpaceUrlCaptureProjection } from "../../sources/url-capture-projection";

type Point = { x: number; y: number };
type AddContent = (data: SpaceNodePayload, position?: Point) => Promise<SpaceCanvasNode[]>;
type SaveNodeContent = (spaceId: string, change: NodePayloadChangeInput) => Promise<void>;

export const URL = "https://www.youtube.com/watch?v=abc123";
export const POSITION = { x: 120, y: 80 };

export function captureHarness() {
  let activeSpaceId = "space-1";
  const projection: SpaceUrlCaptureProjection = {
    id: "import-1",
    spaceId: activeSpaceId,
    url: URL,
    title: "youtube.com",
    position: POSITION,
    status: "resolving",
    downloadedBytes: 0,
    totalBytes: null,
    processedMs: null,
    durationMs: null,
  };
  const begin = vi.fn<(input: Omit<SpaceUrlCaptureProjection, "id">) => SpaceUrlCaptureProjection>(
    () => projection,
  );
  const finish = vi.fn<(id: string) => void>();
  const update = vi.fn<(id: string, patch: object) => void>();
  const addContent = vi.fn<AddContent>(async (data, position = POSITION) => [mediaNode(data, position)]);
  return {
    context: {
      getActiveSpaceId: () => activeSpaceId,
      nodes: [],
      beginUrlCapture: begin,
      updateUrlCapture: update,
      finishUrlCapture: finish,
      addContent,
      saveNodeContentForSpace: vi.fn<SaveNodeContent>(),
    },
    begin,
    finish,
    update,
    addContent,
    setActiveSpace: (spaceId: string) => {
      activeSpaceId = spaceId;
    },
  };
}

export function pendingImport() {
  type CaptureOutcome = { outcome: "imported"; payload: SpaceMediaPayload } | { outcome: "unsupported" };
  let deferredResolve!: (value: CaptureOutcome) => void;
  const promise = new Promise<CaptureOutcome>((resolve) => {
    deferredResolve = resolve;
  });
  return { promise, resolve: deferredResolve };
}

function mediaNode(data: SpaceNodePayload, position: Point): SpaceCanvasNode {
  return { id: "media", type: data.kind, initialPosition: position, data } as SpaceCanvasNode;
}

export function importedVideo(): SpaceMediaPayload {
  return {
    kind: "video",
    title: "Video",
    durationSeconds: 12,
    assetPath: "/spaces/assets/video.mp4",
    fileName: "video.mp4",
    size: 1024,
    modifiedAt: 1,
    sourceRevision: "asset1",
    origin: { kind: "web", url: URL, provider: "Generic", externalId: "42" },
  };
}
