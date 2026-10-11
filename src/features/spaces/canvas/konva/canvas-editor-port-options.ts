import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasHostEvents } from "../contracts/canvas-editor";
import type { CanvasPoint, CanvasRect, CanvasViewport } from "../contracts/canvas-editor-port";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../schema/document";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import type { CanvasPalette } from "./rendering/use-canvas-palette";
import type { CanvasScene } from "./scene/canvas-scene";

export type CanvasFrameMutation = () =>
  | readonly CanvasNodeFrameUpdate[]
  | Promise<readonly CanvasNodeFrameUpdate[]>;

export interface KonvaEditorPortOptions extends CanvasHostEvents {
  animateViewport(viewport: CanvasViewport): void;
  cancelInteraction(): void;
  suspendInteraction(): void;
  runFrameMutation(mutation: CanvasFrameMutation): Promise<void>;
  host(): HTMLElement;
  layouts(): readonly CanvasNodeLayoutV1[];
  lockedNodeIds(): ReadonlySet<string>;
  nudgeSelection(offset: CanvasPoint): Promise<void>;
  pastePoint(): CanvasPoint;
  model: CanvasDocumentModel;
  nodeById(nodeId: string): SpaceCanvasNode | undefined;
  nodes(): readonly SpaceCanvasNode[];
  palette(): CanvasPalette;
  relationIds(): ReadonlySet<string>;
  relationBoundsById(): ReadonlyMap<string, CanvasRect>;
  relations(): readonly SpaceRelation[];
  selectableNodeIds(): readonly string[];
  selectedNodeIds(): ReadonlySet<string>;
  selectedStructureNodeIds(): ReadonlySet<string>;
  scene(): CanvasScene;
  snapshot(): SpaceEditSnapshot;
  structureBoundsById(): ReadonlyMap<string, CanvasRect>;
  setSelection(nodeIds: readonly string[]): void;
  setStructureSelection(structureNodeIds: readonly string[]): void;
  setViewport(viewport: CanvasViewport): void;
  size(): { width: number; height: number };
  viewport(): CanvasViewport;
}
