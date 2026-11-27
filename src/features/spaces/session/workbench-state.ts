import { ref, shallowRef } from "vue";

import type { CanvasBackgroundPattern } from "../../../shared/contracts/generated/CanvasBackgroundPattern";
import type { RelationKind } from "../contracts/generated/RelationKind";
import type { RelationStatus } from "../contracts/generated/RelationStatus";
import type { ConversationScope } from "../conversation/conversation-scope";
import type { SpaceSavedViewFilter } from "../knowledge/model/saved-view-filter";
import type { KnowledgeLocator } from "../knowledge/model/schema/chunk";
import {
  SOURCE_READER_WIDTH_DEFAULT,
  type SourceReadingHistoryEntry,
  type SourceReadingTarget,
} from "../reading/source-reader-target";
import { DEFAULT_RELATION_STATUS_FILTER } from "../workspace/space-view-session";

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

export function createSpaceUiState() {
  return {
    findOpen: ref(false),
    globalFindOpen: ref(false),
    knowledgeHealthOpen: shallowRef(false),
    askScopeRequest: ref<{
      scope: ConversationScope;
      restart: boolean;
      nodeIds: string[];
      quote?: string;
      nonce: number;
    }>(),
    askScopeNonce: ref(0),
    insertionPoint: ref({ x: 96, y: 84 }),
    revealRequest: ref<RevealRequest>(),
    viewportRequest: ref<ViewportRequest>(),
    zoomRequest: ref<ZoomRequest>(),
    selectionClearNonce: ref(0),
    sceneCue: ref<SceneCue>(),
    readingTarget: shallowRef<SourceReadingTarget>(),
    readingHistory: shallowRef<readonly SourceReadingHistoryEntry[]>([]),
    readingHistoryIndex: shallowRef(-1),
    readingFollowsSelection: shallowRef(false),
    readerWidth: shallowRef(SOURCE_READER_WIDTH_DEFAULT),
    readerFocusMode: shallowRef(false),
    backgroundPattern: shallowRef<CanvasBackgroundPattern>(),
    editorLocateRequest: ref<EditorLocateRequest>(),
    canvasFocusIds: ref<string[]>([]),
    activeSceneId: ref<string | undefined>(),
    outlineExpandedIds: shallowRef<string[]>([]),
    outlineFocusRequest: shallowRef(0),
    outlineRenameStructureNodeId: shallowRef<string | null>(null),
    thumbnailDirty: ref(true),
    relationKindFilter: ref<RelationKind[]>([]),
    relationStatusFilter: ref<RelationStatus[]>([...DEFAULT_RELATION_STATUS_FILTER]),
    savedQueryFilter: ref<SpaceSavedViewFilter>({}),
    minimapOpen: ref(false),
  };
}
