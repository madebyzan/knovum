import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { StructureCommand } from "../../organization/structure/commands/contract";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../contract/canvas-document";
import type { CanvasSurfaceEvents } from "../contract/canvas-editor";
import type { CanvasPoint, CanvasRect, CanvasViewport } from "../contract/canvas-editor-port";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { CanvasPalette } from "./presentation/use-canvas-palette";

export type CanvasFrameMutation = () =>
  | readonly CanvasNodeFrameUpdate[]
  | Promise<readonly CanvasNodeFrameUpdate[]>;

export interface KonvaEditorPortOptions extends CanvasSurfaceEvents {
  animateViewport(viewport: CanvasViewport): void;
  cancelInteraction(): void;
  executeStructureCommand(command: StructureCommand): Promise<void>;
  enterGroup(id: string): void;
  exitGroup(): void;
  scopeGroupId(): string | null;
  setEditingScope(ids: readonly string[]): void;
  requestContainerRename(id: string): void;
  runFrameMutation(mutation: CanvasFrameMutation): Promise<void>;
  host(): HTMLElement;
  layouts(): readonly CanvasNodeLayoutV1[];
  nudgeSelection(offset: CanvasPoint): Promise<void>;
  pastePoint(): CanvasPoint;
  model: CanvasDocumentModel;
  nodeById(id: string): SpaceCanvasNode | undefined;
  nodes(): readonly SpaceCanvasNode[];
  palette(): CanvasPalette;
  relationIds(): ReadonlySet<string>;
  relationBounds(): ReadonlyMap<string, CanvasRect>;
  relations(): readonly SpaceRelation[];
  selectableNodeIds(): readonly string[];
  selectedNodeIds(): ReadonlySet<string>;
  setSelection(ids: readonly string[]): void;
  setViewport(viewport: CanvasViewport): void;
  size(): { width: number; height: number };
  viewport(): CanvasViewport;
}
