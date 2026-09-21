import type { Ref } from "vue";

import type { SpaceEditSnapshot } from "../canvas/contract/space-edit-snapshot";
import type { SpaceHistoryState } from "../history/stack";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space } from "../model/space";
import type { SpaceScene } from "../scenes/scene";

export interface SpaceSessionContext {
  spaces: Ref<Space[]>;
  draftSpace: Ref<Space | undefined>;
  activeSpaceId: Ref<string | undefined>;
  nodes: Ref<SpaceCanvasNode[]>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  groupScopePath: Ref<string[]>;
  scenes: Ref<SpaceScene[]>;
  relations: Ref<SpaceRelation[]>;
  ready: Ref<boolean>;
  loading: Ref<boolean>;
  focusedEditorId: Ref<string | undefined>;
  history: SpaceHistoryState;
  runtime: { openGeneration: number; pendingSpaceId?: string | undefined };
  openSpaceIds: Ref<string[]>;
  closedSpaceIds: Ref<string[]>;
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

export interface LoadSpaceOptions {
  markOpened: boolean;
  resetHistory: boolean;
}
