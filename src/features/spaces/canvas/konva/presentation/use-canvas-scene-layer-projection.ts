import { computed } from "vue";

import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasScene, CanvasSceneNode } from "../scene/canvas-scene";

interface CanvasSceneLayerProjectionInput {
  connectorDraft: CanvasConnectorDraft | undefined;
  scene: CanvasScene;
  selectedNodeIds: ReadonlySet<string>;
  visibleNodeIds: ReadonlySet<string>;
  visibleRelationIds: ReadonlySet<string>;
}

export function useCanvasSceneLayerProjection(props: CanvasSceneLayerProjectionInput) {
  const relationsByKey = computed(
    () => new Map(props.scene.relations.map((relation, order) => [relation.key, { relation, order }])),
  );
  const canonicalVisibleRelations = computed(() =>
    [...props.visibleRelationIds]
      .flatMap((key) => {
        const indexed = relationsByKey.value.get(key);
        return indexed ? [indexed] : [];
      })
      .toSorted((left, right) => left.order - right.order)
      .map(({ relation }) => relation),
  );
  const selectedNodes = computed(() => sceneNodesInPaintOrder(props.scene, props.selectedNodeIds));
  const selectedFrames = computed(() => selectedNodes.value.map(({ layout }) => layout.frame));
  return {
    visibleRelations: canonicalVisibleRelations,
    visibleContentNodes: computed(() => sceneNodesInPaintOrder(props.scene, props.visibleNodeIds)),
    connectorNode: computed(() => {
      if (props.selectedNodeIds.size !== 1) return undefined;
      return selectedNodes.value.find(({ interaction }) => interaction.connectable);
    }),
    connectorTargetNode: computed(() =>
      props.connectorDraft?.target
        ? props.scene.nodesById.get(props.connectorDraft.target.nodeId)
        : undefined,
    ),
    resizeSnapFrames: computed(() =>
      props.scene.nodes
        .filter(({ node }) => !props.selectedNodeIds.has(node.id))
        .map(({ layout }) => layout.frame),
    ),
    selectedFrames,
  };
}

function sceneNodesInPaintOrder(scene: CanvasScene, ids: ReadonlySet<string>): CanvasSceneNode[] {
  return [...ids]
    .flatMap((id) => {
      const node = scene.nodesById.get(id);
      return node ? [node] : [];
    })
    .toSorted((left, right) => left.paintOrder - right.paintOrder);
}
