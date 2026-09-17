import type { SpaceCanvasDetailLevel } from "../../assets/preview-policy";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "./canvas-document";
import type { CanvasSurfaceEvents } from "./canvas-editor";
import type { CanvasTool } from "./canvas-tool";

export interface CanvasSurfaceProps {
  document: CanvasDocumentV1;
  events: CanvasSurfaceEvents;
  focusNodeIds: readonly string[];
  interactionEnabled: boolean;
  tool: CanvasTool;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
}

export interface CanvasNodeContentSlotProps {
  interactive: boolean;
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  detailLevel: SpaceCanvasDetailLevel;
  selected: boolean;
}
