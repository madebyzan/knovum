import { ref, shallowRef } from "vue";

import type { ConversationScope } from "../conversation/conversation-scope";
import type { SpaceTiptapHeading } from "../editor/tiptap-outline";
import type { KnowledgePeekTarget } from "../knowledge/model/knowledge-peek-target";
import type { SpaceSavedViewFilter } from "../knowledge/model/projections";
import type { KnowledgeLocator } from "../knowledge/model/schema";
import type { SpaceRelationKind, SpaceRelationStatus } from "../model/relations/relation";

export interface RevealRequest {
  ids: string[];
  nonce: number;
  mode: "ensure-visible" | "focus";
}

export interface ViewportRequest {
  viewport: { x: number; y: number; zoom: number };
  nonce: number;
}

export type CanvasZoomMode = "in" | "out" | "selection" | "fit";

export interface ZoomRequest {
  mode: CanvasZoomMode;
  nonce: number;
}

interface EditorLocateRequest {
  nodeId: string;
  locator: KnowledgeLocator;
  nonce: number;
}

export interface SceneCue {
  sceneId: string;
  title: string;
  action: "saved" | "opened" | "updated";
  zoom: number;
  nonce: number;
}

export type RightSidebarView = "ask" | "inspect" | "document-outline" | "source";
export type RightSidebarPrimaryView = Exclude<RightSidebarView, "source">;

export function createSpaceUiState() {
  return {
    findOpen: ref(false),
    globalFindOpen: ref(false),
    askScopeRequest: ref<{ scope: ConversationScope; restart: boolean; nodeIds: string[]; nonce: number }>(),
    askScopeNonce: ref(0),
    rightSidebarView: ref<RightSidebarView>("ask"),
    workflowMode: shallowRef<"capture" | "use">(),
    documentOutlineHeadings: shallowRef<SpaceTiptapHeading[]>([]),
    documentActiveHeadingId: shallowRef<string>(),
    documentOutlineRequest: shallowRef<{ position: number; nonce: number }>(),
    workflowCaptureSelection: ref<string[]>([]),
    focusedEditorId: ref<string>(),
    videoEditorNodeId: ref<string>(),
    insertionPoint: ref({ x: 96, y: 84 }),
    revealRequest: ref<RevealRequest>(),
    viewportRequest: ref<ViewportRequest>(),
    zoomRequest: ref<ZoomRequest>(),
    selectionClearNonce: ref(0),
    sceneCue: ref<SceneCue>(),
    sourcePeek: ref<KnowledgePeekTarget>(),
    sourceReturnView: ref<RightSidebarPrimaryView>("ask"),
    editorLocateRequest: ref<EditorLocateRequest>(),
    canvasFocusIds: ref<string[]>([]),
    outlineFocusRequest: shallowRef(0),
    outlineRenameStructureNodeId: shallowRef<string | null>(null),
    recentlyDeletedSpaceId: shallowRef<string | null>(),
    thumbnailDirty: ref(true),
    relationKindFilter: ref<SpaceRelationKind[]>([]),
    relationStatusFilter: ref<SpaceRelationStatus[]>(["active", "contested"]),
    savedQueryFilter: ref<SpaceSavedViewFilter>({}),
  };
}
