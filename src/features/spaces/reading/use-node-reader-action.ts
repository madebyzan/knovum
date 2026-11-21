import { computed, toValue, type MaybeRefOrGetter } from "vue";

import { useSpaceProvenanceQuery } from "../knowledge/queries/use-space-provenance-query";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { useSpaceStore } from "../session/space-store";
import { isReadableSourceNode } from "./source-reader-target";

export interface NodeReaderAction {
  mode: "read" | "locate-source";
  labelKey: "spaces.inspector.reader.read" | "spaces.conversation.sourcePeek.open";
  run: () => void;
}

/**
 * Resolves the primary reading action for a canvas node:
 * - Any readable node (PDF, Word, Excel, Slides, Website, Media, Code, Markdown, Note) opens itself in
 *   the Source Reader so clicking Read on a node always displays that node's own content.
 * - Non-readable derived nodes (such as Knowledge Objects) jump to their active cited source passage.
 */
export function useNodeReaderAction(node: MaybeRefOrGetter<SpaceCanvasNode | undefined>) {
  const store = useSpaceStore();
  const currentNode = computed(() => toValue(node));
  const isReadable = computed(() => isReadableSourceNode(currentNode.value));
  const provenanceQuery = useSpaceProvenanceQuery(
    () => store.activeSpaceId,
    "node",
    () => (!isReadable.value && currentNode.value ? currentNode.value.id : ""),
  );

  const primaryProvenance = computed(() => {
    if (isReadable.value) return undefined;
    const records = provenanceQuery.data.value ?? [];
    for (const record of records) {
      if (record.status !== "active") continue;
      const sourceNode = store.nodes.find(
        (candidate) => candidate.id === record.sourceNodeId && !candidate.hidden,
      );
      if (isReadableSourceNode(sourceNode)) return record;
    }
    return undefined;
  });

  return computed<NodeReaderAction | undefined>(() => {
    const target = currentNode.value;
    if (!target) return undefined;
    if (isReadable.value) {
      return {
        mode: "read",
        labelKey: "spaces.inspector.reader.read",
        run: () => store.openSourceReader({ nodeId: target.id }),
      };
    }
    const record = primaryProvenance.value;
    if (!record) return undefined;
    return {
      mode: "locate-source",
      labelKey: "spaces.conversation.sourcePeek.open",
      run: () =>
        store.openSourceReader({
          nodeId: record.sourceNodeId,
          locator: record.locator,
          quote: record.quote,
        }),
    };
  });
}
