import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import type { SpaceProvenance } from "../../knowledge/model/provenance";
import { listProvenance } from "../../knowledge/persistence/knowledge-operations";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodeKind } from "../../model/nodes/node-payload";
import type { PdfNormalizedRect } from "../../reading/pdf-rect-geometry";
import { useSpaceStore } from "../../session/space-store";

export interface PdfProvenanceLink {
  id: string;
  targetNodeId: string;
  targetTitle: string;
  targetKind: SpaceNodeKind;
  page: number;
  rects: readonly PdfNormalizedRect[];
  quote: string;
}

function toPdfLink(
  record: SpaceProvenance,
  sourceId: string,
  nodesById: ReadonlyMap<string, SpaceCanvasNode>,
): PdfProvenanceLink | undefined {
  if (
    record.status !== "active" ||
    record.sourceNodeId !== sourceId ||
    record.locator.location.kind !== "pdf"
  ) {
    return undefined;
  }
  const target = nodesById.get(record.targetId);
  if (!target || target.hidden) return undefined;
  return {
    id: record.id,
    targetNodeId: target.id,
    targetTitle: target.data.title,
    targetKind: target.data.kind,
    page: record.locator.location.page,
    rects: record.locator.location.rects ?? [],
    quote: record.quote,
  };
}

/** Loads all active Canvas provenance links derived from a PDF node so the PDF reader can paint them live. */
export function usePdfSourceLinks(pdfNodeId: MaybeRefOrGetter<string | undefined>) {
  const store = useSpaceStore();
  const nodeId = computed(() => toValue(pdfNodeId));

  const derivedTargetIds = computed(() => {
    const sourceId = nodeId.value;
    if (!sourceId) return [];
    const ids = new Set<string>();
    for (const rel of store.relations) {
      if (rel.kind === "derives-from" && rel.status !== "invalidated" && rel.targetId === sourceId) {
        ids.add(rel.sourceId);
      }
    }
    const visibleIds = new Set(store.nodes.filter((n) => !n.hidden).map((n) => n.id));
    return [...ids].filter((id) => visibleIds.has(id)).toSorted();
  });

  const query = useQuery(
    computed(() => {
      const spaceId = store.activeSpaceId;
      const sourceId = nodeId.value;
      const targets = derivedTargetIds.value;
      return {
        queryKey: ["spaces", spaceId ?? "", "pdf-source-links", sourceId ?? "", targets.join(",")] as const,
        enabled: Boolean(spaceId && sourceId && targets.length),
        queryFn: async (): Promise<readonly PdfProvenanceLink[]> => {
          if (!spaceId || !sourceId || !targets.length) return [];
          const lists = await Promise.all(targets.map((id) => listProvenance(spaceId, "node", id)));
          const nodesById = new Map(store.nodes.map((node) => [node.id, node]));
          const results = lists
            .flat()
            .map((record) => toPdfLink(record, sourceId, nodesById))
            .filter((item): item is PdfProvenanceLink => item !== undefined);
          return results.toSorted((a, b) =>
            a.page !== b.page ? a.page - b.page : (a.rects[0]?.y ?? 0) - (b.rects[0]?.y ?? 0),
          );
        },
      };
    }),
  );

  const links = computed<readonly PdfProvenanceLink[]>(() => query.data.value ?? []);
  const highlightedPages = computed(() => new Set(links.value.map((link) => link.page)));

  return { links, highlightedPages };
}
