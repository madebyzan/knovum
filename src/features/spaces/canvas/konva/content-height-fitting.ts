import type { StructureLayout } from "../../model/structure/schema/structure";
import { createStructureIndex } from "../../model/structure/structure-index";
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

  function commit(heights: ReadonlyMap<string, FittedContentHeight>) {
    if (!heights.size) return;
    const submission = commitTail.then(() => {
      const snapshot = options.snapshot();
      const index = createStructureIndex(snapshot.structure);
      const layouts = new Map(snapshot.layouts.map((layout) => [layout.structureNodeId, layout]));
      const updates = [...heights].flatMap(([nodeId, measurement]) =>
        update(nodeId, measurement, index, layouts),
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
    layouts: ReadonlyMap<string, StructureLayout>,
  ) {
    const node = options.nodeById(nodeId);
    const structureNodeId = index.nodeIdToStructureNodeId.get(nodeId);
    const layout = structureNodeId ? layouts.get(structureNodeId) : undefined;
    if (!node || !structureNodeId || layout?.kind !== "content") return [];
    const matchesSource = [
      (node.data.knowledgeRevision ?? null) === measurement.contentRevision,
      layout.sizingMode === measurement.sizingMode,
      layout.size.width === measurement.measuredWidth,
      (layout.appearance?.typographyScale ?? 1) === measurement.typographyScale,
    ].every(Boolean);
    return matchesSource ? [{ structureNodeId, ...measurement }] : [];
  }
}
