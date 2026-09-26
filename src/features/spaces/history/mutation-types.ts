import type { Ref } from "vue";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceHistoryState } from "./stack";

export interface SpaceMutationContext {
  nodes: Ref<SpaceCanvasNode[]>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  groupScopePath: Ref<string[]>;
  activeSpaceId: Ref<string | undefined>;
  history: SpaceHistoryState;
  nextInsertionPoint: () => { x: number; y: number };
  ensureActiveSpacePersisted: () => Promise<void>;
  recordCommittedSpaceEdit: () => void;
}

export interface NodePayloadChange {
  id: string;
  before: SpaceNodePayload;
  after: SpaceNodePayload;
}

export function requireActiveSpaceId(activeSpaceId: Ref<string | undefined>) {
  if (!activeSpaceId.value) throw new Error("Space requires an active space");
  return activeSpaceId.value;
}

export function assertSpacePosition(position: { x: number; y: number }) {
  if (!Number.isFinite(position.x) || !Number.isFinite(position.y)) {
    throw new Error("Space node position must be finite");
  }
}
