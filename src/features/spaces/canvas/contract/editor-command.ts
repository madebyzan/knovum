import type { StructureCommand } from "../../organization/structure/commands/contract";
import type {
  CanvasNodeAppearance,
  CanvasNodeFrameUpdate,
  CanvasRelationOverrideColor,
  SpaceEditorDocument,
} from "./canvas-document";
import type { CanvasLayerMove } from "./canvas-editor-port";
import type { EditorFragment, FragmentPlacement } from "./editor-fragment";

export interface EditorFrameUpdate extends CanvasNodeFrameUpdate {
  typographyScale?: number;
}
export interface EditorTextMeasurement {
  itemId: string;
  width: number;
  height: number;
  typographyScale: number;
  contentRevision: string;
}
export type SpaceEditorCommand =
  | { kind: "insert-fragment"; fragment: EditorFragment; placement: FragmentPlacement }
  | { kind: "structure"; command: StructureCommand; selection: readonly string[] }
  | { kind: "transform-frames" | "arrange-frames"; updates: readonly EditorFrameUpdate[] }
  | {
      kind: "set-appearance";
      itemId: string;
      changes: Partial<{ [K in keyof CanvasNodeAppearance]: CanvasNodeAppearance[K] | null }>;
    }
  | { kind: "set-relation-color"; relationId: string; color: CanvasRelationOverrideColor | null }
  | { kind: "move-layer"; ids: readonly string[]; mode: CanvasLayerMove }
  | { kind: "delete-selection"; ids: readonly string[]; relationIds: readonly string[] }
  | { kind: "measure-text"; measurements: readonly EditorTextMeasurement[] };

export interface ExecuteSpaceEditRequest {
  baseRevision: number;
  mutationId: string;
  command: SpaceEditorCommand;
}
export interface ExecuteSpaceEditDraft {
  baseDocument: SpaceEditorDocument;
  mutationId: string;
  command: SpaceEditorCommand;
}
export interface ReplaySpaceEditRequest {
  baseRevision: number;
  mutationId: string;
  receiptId: string;
  direction: "undo" | "redo";
}
export type ReplaySpaceEditDraft = Omit<ReplaySpaceEditRequest, "baseRevision">;
