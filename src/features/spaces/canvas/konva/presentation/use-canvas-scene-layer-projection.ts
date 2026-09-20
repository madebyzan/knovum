import { computed } from "vue";

import type { CanvasConnectorDraft } from "../interaction/connector-gesture-controller";
import type { CanvasScene } from "../scene/scene-projection";

interface CanvasSceneLayerProjectionInput {
  connectorDraft: CanvasConnectorDraft | undefined;
  scene: CanvasScene;
  selectedNodeIds: ReadonlySet<string>;
  visibleNodeIds: ReadonlySet<string>;
  visibleRelationIds: ReadonlySet<string>;
}

export function useCanvasSceneLayerProjection(props: CanvasSceneLayerProjectionInput) {
  const canonicalVisibleRelations = computed(() =>
    props.scene.relations.filter(({ key }) => props.visibleRelationIds.has(key)),
  );
  const selectedLayouts = computed(() =>
    props.scene.nodes.filter(({ node }) => props.selectedNodeIds.has(node.id)).map(({ layout }) => layout),
  );
  return {
    visibleRelations: canonicalVisibleRelations,
    visibleSections: computed(() =>
      props.scene.sections.filter(({ node }) => props.visibleNodeIds.has(node.id)),
    ),
    visibleContentNodes: computed(() =>
      props.scene.contentNodes.filter(({ node }) => props.visibleNodeIds.has(node.id)),
    ),
    connectorNode: computed(() => {
      if (props.selectedNodeIds.size !== 1) return undefined;
      return props.scene.nodes.find(
        ({ interaction, node }) => interaction.connectable && props.selectedNodeIds.has(node.id),
      );
    }),
    connectorTargetNode: computed(() =>
      props.connectorDraft?.target
        ? props.scene.nodes.find(({ node }) => node.id === props.connectorDraft?.target?.nodeId)
        : undefined,
    ),
    resizeSnapFrames: computed(() =>
      props.scene.nodes
        .filter(({ node }) => !props.selectedNodeIds.has(node.id))
        .map(({ layout }) => layout.frame),
    ),
    selectedFrames: computed(() => selectedLayouts.value.map((layout) => layout.frame)),
  };
}
