import type { StructureLayout } from "../../model/structure/schema/structure";
import { createStructureIndex } from "../../model/structure/structure-index";
import { worldTransformFor } from "../../model/structure/world-transform";
import type { KonvaEditorPortOptions } from "./canvas-editor-port-options";
import { createNodeContentHeightBatch, type FittedContentHeight } from "./node-content-height-batch";

type FittingOptions = Pick<
  KonvaEditorPortOptions,
  "nodeById" | "scene" | "snapshot" | "onExecuteSpaceEdit" | "onError"
>;

export function createCanvasContentHeightFitting(options: FittingOptions) {
  let commitTail: Promise<unknown> = Promise.resolve();
  return createNodeContentHeightBatch(
    options.nodeById,
    (id) => options.scene().nodesById.get(id)?.layout,
    commit,
  );

  function commit(heightByNodeId: ReadonlyMap<string, FittedContentHeight>) {
    if (!heightByNodeId.size) return;
    const submission = commitTail.then(() => {
      const snapshot = options.snapshot();
      const index = createStructureIndex(snapshot.structure);
      const layoutByStructureNodeId = new Map(
        snapshot.layouts.map((layout) => [layout.structureNodeId, layout]),
      );
      const updates = [...heightByNodeId].flatMap(([nodeId, measurement]) =>
        update(nodeId, measurement, index, layoutByStructureNodeId),
      );
      return updates.length
        ? options.onExecuteSpaceEdit({ kind: "content.update-heights", updates })
        : undefined;
    });
    commitTail = submission.catch(() => undefined);
    void submission.catch(options.onError);
  }

  function update(
    nodeId: string,
    measurement: FittedContentHeight,
    index: ReturnType<typeof createStructureIndex>,
    layoutByStructureNodeId: ReadonlyMap<string, StructureLayout>,
  ) {
    const node = options.nodeById(nodeId);
    const structureNodeId = index.structureNodeIdByNodeId.get(nodeId);
    const layout = structureNodeId ? layoutByStructureNodeId.get(structureNodeId) : undefined;
    if (!node || !structureNodeId || layout?.kind !== "content") return [];
    const worldScale = worldTransformFor(structureNodeId, index, layoutByStructureNodeId).scale;
    const height = measurement.height / worldScale;
    const shouldSubmit = [
      (node.data.knowledgeRevision ?? null) === measurement.contentRevision,
      layout.sizingMode === measurement.sizingMode,
      options.scene().nodesById.has(nodeId),
      layout.size.width * worldScale === measurement.measuredWidth,
      (layout.appearance?.typographyScale ?? 1) === measurement.typographyScale,
      [layout.sizingMode === "content", layout.size.height !== height].some(Boolean),
    ].every(Boolean);
    return shouldSubmit
      ? [{ structureNodeId, ...measurement, height, measuredWidth: layout.size.width }]
      : [];
  }
}
