import { vi } from "vitest";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceMediaPayload } from "../../model/nodes/capabilities";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { NodePayloadChangeInput } from "../../nodes/node-operations";

type Point = { x: number; y: number };
type CreatePending = (
  spaceId: string,
  data: SpaceNodePayload,
  position?: Point,
) => Promise<SpaceCanvasNode[]>;
type SavePending = (spaceId: string, change: NodePayloadChangeInput) => Promise<void>;
type Finalize = (spaceId: string, nodes: readonly SpaceCanvasNode[], reveal?: boolean) => void;
type Discard = (spaceId: string, nodeId: string) => Promise<void>;
type AddContent = (data: SpaceNodePayload, position?: Point) => Promise<SpaceCanvasNode[]>;

export const URL = "https://www.youtube.com/watch?v=abc123";
export const POSITION = { x: 120, y: 80 };
export const RESERVED_ASSET = "/spaces/assets/import-1.mp4";
/** What the canvas calls media that has not introduced itself yet. */
export const ARRIVING_TITLE = "www.youtube.com/watch?v=abc123";

export function captureHarness() {
  let activeSpaceId = "space-1";
  const createPending = vi.fn<CreatePending>(async (_spaceId, data, position = POSITION) => [
    mediaNode(data, position),
  ]);
  const savePending = vi.fn<SavePending>();
  const finalize = vi.fn<Finalize>();
  const discard = vi.fn<Discard>();
  return {
    context: {
      getActiveSpaceId: () => activeSpaceId,
      nodes: [],
      createPendingContentForSpace: createPending,
      savePendingNodeContentForSpace: savePending,
      finalizePendingContent: finalize,
      discardPendingContent: discard,
      addContent: vi.fn<AddContent>(),
      saveNodeContentForSpace: vi.fn<SavePending>(),
    },
    createPending,
    savePending,
    finalize,
    discard,
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
    assetPath: RESERVED_ASSET,
    fileName: "import-1.mp4",
    size: 1024,
    modifiedAt: 1,
    sourceRevision: "asset1",
    origin: { kind: "web", url: URL, provider: "Generic", externalId: "42" },
  };
}
