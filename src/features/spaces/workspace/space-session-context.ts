import type { Ref } from "vue";

import type { ReversibleSpaceChange, SpaceHistoryState } from "../history/stack";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { Space } from "../model/space";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import type { SpaceScene } from "../scenes/scene";
import type { SpaceSessionContent } from "../session/space-projection";
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
    /** The sessions of open Spaces other than the active one, keyed by Space id. */
    retained: Map<string, RetainedSpaceSession>;
  };
  navigationHistory: Ref<string[]>;
  navigationIndex: Ref<number>;
}

/**
 * What an open Space keeps while another Space is active: the content it last showed and its undo
 * history. Returning to it shows that content at once; it is then revalidated in the background,
 * because work may have published into the Space meanwhile. Released with the Space's last tab.
 */
export interface RetainedSpaceSession {
  content: SpaceSessionContent;
  undoStack: readonly ReversibleSpaceChange[];
  redoStack: readonly ReversibleSpaceChange[];
}
