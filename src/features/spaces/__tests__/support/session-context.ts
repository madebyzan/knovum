import { ref } from "vue";

import type { Space } from "../../model/space";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import { createSpaceWorkItems, selectSpaceWorkItem, type SpaceEditorTab } from "../../workspace/work-items";

export function createTestSpace(overrides: Partial<Space> = {}): Space {
  return {
    id: "inbox",
    kind: "inbox",
    title: "Inbox",
    archivedAt: null,
    favorite: false,
    thumbnailPath: null,
    nodeCount: 0,
    lastOpenedAt: 1,
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

function defaultSessionContext(activeSpaceId = ref<string | undefined>("inbox")): SpaceSessionContext {
  return {
    ...createSpaceWorkItems(activeSpaceId),
    editors: { open: () => {}, release: () => {}, canDiscard: () => false },
    captureOverview: () => undefined,
    spaces: ref([createTestSpace()]),
    draftSpace: ref(),
    activeSpaceId,
    nodes: ref([]),
    editSnapshot: ref({ editRevision: 0, structure: [], layouts: [], relationStyles: [] }),
    groupScopePath: ref([]),
    scenes: ref([]),
    relations: ref([]),
    ready: ref(true),
    loading: ref(false),
    history: { undoStack: [], redoStack: [], busy: false, pendingMutations: 0 },
    runtime: { openGeneration: 0, historyBySpace: new Map() },
    openSpaceIds: ref(["inbox"]),
    closedSpaceIds: ref([]),
    navigationHistory: ref(["inbox"]),
    navigationIndex: ref(0),
  };
}

export function createTestSessionContext(overrides: Partial<SpaceSessionContext> = {}): SpaceSessionContext {
  return Object.assign(defaultSessionContext(overrides.activeSpaceId), overrides);
}

export function activateTestEditor(
  context: SpaceSessionContext,
  kind: SpaceEditorTab["kind"],
  nodeId: string,
) {
  const spaceId = context.activeSpaceId.value!;
  const tab = { id: `${spaceId}:${kind}:${nodeId}`, spaceId, kind, nodeId };
  context.editorTabs.value = [...context.editorTabs.value, tab];
  selectSpaceWorkItem(context, spaceId, tab.id);
  return tab;
}
