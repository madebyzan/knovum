import type { Ref } from "vue";

import type { ReversibleSpaceChange, SpaceHistoryState } from "../history/stack";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space } from "../model/space";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { SpaceScene } from "../scenes/scene";
import type { createEditorSessions } from "./editor-sessions";
import type { SpaceWorkbenchTabState } from "./workbench-tab-state";

export interface SpaceSessionContext extends SpaceWorkbenchTabState {
  editors: ReturnType<typeof createEditorSessions>;
  captureOverview(): void;
  spaces: Ref<Space[]>;
  draftSpace: Ref<Space | undefined>;
  nodes: Ref<SpaceCanvasNode[]>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  groupScopePath: Ref<string[]>;
  scenes: Ref<SpaceScene[]>;
  relations: Ref<SpaceRelation[]>;
  ready: Ref<boolean>;
  loading: Ref<boolean>;
  history: SpaceHistoryState;
  runtime: {
    openGeneration: number;
    pendingSpaceId?: string | undefined;
    historyBySpace: Map<
      string,
      { undoStack: readonly ReversibleSpaceChange[]; redoStack: readonly ReversibleSpaceChange[] }
    >;
  };
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

export interface LoadSpaceOptions {
  markOpened: boolean;
  resetHistory: boolean;
}
