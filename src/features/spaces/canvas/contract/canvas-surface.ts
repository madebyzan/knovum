import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/edit-snapshot";
import type { CanvasViewportDetailProfile } from "./canvas-detail-profile";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "./canvas-document";
import type { CanvasSurfaceEvents } from "./canvas-editor";
import type { CanvasTool } from "./canvas-tool";

export interface CanvasSceneInput {
  editSnapshot: SpaceEditSnapshot;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
}

export interface CanvasSurfaceProps {
  document: CanvasDocumentV1;
  events: CanvasSurfaceEvents;
  focusNodeIds: readonly string[];
  groupScopePath: readonly string[];
  interactionEnabled: boolean;
  tool: CanvasTool;
  sceneInput: CanvasSceneInput;
}

export interface CanvasNodeContentSlotProps {
  interactive: boolean;
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  detailProfile: CanvasViewportDetailProfile;
  selected: boolean;
}
