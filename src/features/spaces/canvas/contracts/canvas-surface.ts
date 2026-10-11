import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import type { CanvasPreviewPolicy } from "../preview/policy";
import type { CanvasDocumentV1, CanvasNodeLayoutV1 } from "../schema/document";
import type { CanvasHostEvents } from "./canvas-editor";
import type { CanvasTool } from "./canvas-tool";

export interface CanvasSceneInput {
  editSnapshot: SpaceEditSnapshot;
  nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[];
}

export interface CanvasSurfaceProps {
  document: CanvasDocumentV1;
  events: CanvasHostEvents;
  focusNodeIds: readonly string[];
  groupScopePath: readonly string[];
  isInteractionEnabled: boolean;
  isNavigationEnabled: boolean;
  isRenderingEnabled: boolean;
  canCommitCanvasGestures: boolean;
  tool: CanvasTool;
  sceneInput: CanvasSceneInput;
}

export interface CanvasNodeContentSlotProps {
  interactive: boolean;
  layout: CanvasNodeLayoutV1;
  node: SpaceCanvasNode;
  previewPolicy: CanvasPreviewPolicy;
  selected: boolean;
}
