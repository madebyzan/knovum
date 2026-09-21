import type Konva from "konva";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "./canvas-gesture-update";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { scaledCanvasTextAppearance } from "./canvas-text-transform";

export interface CanvasTransformTarget {
  content: SpaceCanvasNode;
  layout: CanvasNodeLayoutV1;
  node: Konva.Node;
}

export function captureCanvasTransformTargets(
  layouts: readonly CanvasNodeLayoutV1[],
  nodes: readonly SpaceCanvasNode[],
  selectedNodeIds: ReadonlySet<string>,
  stage: Konva.Stage | undefined,
) {
  const contentById = new Map(nodes.map((node) => [node.id, node]));
  const sceneNodes = captureCanvasNodeRefs(stage, selectedNodeIds);
  return layouts.flatMap<CanvasTransformTarget>((layout) => {
    if (!selectedNodeIds.has(layout.nodeId)) return [];
    const content = contentById.get(layout.nodeId);
    const node = sceneNodes.get(layout.nodeId);
    return content && node ? [{ content, layout, node }] : [];
  });
}

interface CanvasTransformProjectionOptions {
  proportionalAnchor: string | undefined;
  targets: readonly CanvasTransformTarget[];
}

export function projectCanvasTransformUpdates(
  options: CanvasTransformProjectionOptions,
): CanvasGestureUpdate[] {
  return options.targets.flatMap<CanvasGestureUpdate>(({ content, layout, node }) => {
    const nodeId = content.id;
    const frame = {
      x: node.x(),
      y: node.y(),
      width: Math.max(1, layout.frame.width * Math.abs(node.scaleX())),
      height: Math.max(1, layout.frame.height * Math.abs(node.scaleY())),
      rotation: node.rotation(),
    };
    const scalesText = content.data.kind === "text" && options.proportionalAnchor === "bottom-right";
    return [
      {
        nodeId,
        frame,
        sizeAuthority: "frame",
        ...(scalesText ? { appearance: scaledCanvasTextAppearance(layout, frame.width) } : {}),
      },
    ];
  });
}
