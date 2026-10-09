import type { Ref } from "vue";

import type { ReversibleSpaceChange, SpaceHistoryState } from "../history/stack";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space } from "../model/space";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { SpaceScene } from "../scenes/scene";

export interface SpaceSessionContext {
  captureOverview(): void;
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
  runtime: {
    openGeneration: number;
    pendingSpaceId?: string | undefined;
    historyBySpace: Map<
      string,
      { undoStack: readonly ReversibleSpaceChange[]; redoStack: readonly ReversibleSpaceChange[] }
    >;
  };
  openSpaceIds: Ref<string[]>;
  closedSpaceIds: Ref<string[]>;
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

export interface LoadSpaceOptions {
  markOpened: boolean;
  resetHistory: boolean;
}
