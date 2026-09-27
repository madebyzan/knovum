import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasSurfaceEvents } from "../contracts/canvas-editor";
import type { CanvasPoint, CanvasRect, CanvasViewport } from "../contracts/canvas-editor-port";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../schema/document";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { CanvasPalette } from "./presentation/use-canvas-palette";
import type { CanvasScene } from "./scene/scene-projection";

export type CanvasFrameMutation = () =>
  | readonly CanvasNodeFrameUpdate[]
  | Promise<readonly CanvasNodeFrameUpdate[]>;

export interface KonvaEditorPortOptions extends CanvasSurfaceEvents {
  animateViewport(viewport: CanvasViewport): void;
  cancelInteraction(): void;
  runFrameMutation(mutation: CanvasFrameMutation): Promise<void>;
  host(): HTMLElement;
  layouts(): readonly CanvasNodeLayoutV1[];
  lockedNodeIds(): ReadonlySet<string>;
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
  selectedStructureNodeIds(): ReadonlySet<string>;
  scene(): CanvasScene;
  snapshot(): SpaceEditSnapshot;
  structureBounds(): ReadonlyMap<string, CanvasRect>;
  setSelection(ids: readonly string[]): void;
  setStructureSelection(ids: readonly string[]): void;
  setViewport(viewport: CanvasViewport): void;
  size(): { width: number; height: number };
  viewport(): CanvasViewport;
}
