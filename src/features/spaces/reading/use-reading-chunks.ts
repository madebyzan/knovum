import { useQuery } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import { useRecognizedText } from "../enrichment/use-recognized-text";
import { listCurrentKnowledgeChunks } from "../knowledge/persistence/chunk-operations";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SourceReaderKind } from "./source-reader-target";

const CHUNK_READER_KINDS = new Set<SourceReaderKind>(["slides", "document", "code", "image"]);
const OCR_READER_KINDS = new Set<SourceReaderKind>(["pdf", "slides", "image"]);

function collectRecognizedPdfPages(
  occurrences: readonly { location: { kind: string; page?: number }; resourceId: string | null }[],
  recognizedText: (resourceId: string) => string | undefined,
): Map<number, readonly string[]> {
  const byPage = new Map<number, string[]>();
  for (const occurrence of occurrences) {
    if (
      occurrence.location.kind !== "pdf" ||
      occurrence.location.page === undefined ||
      !occurrence.resourceId
    )
      continue;
    const passage = recognizedText(occurrence.resourceId)?.trim();
    if (!passage) continue;
    const existing = byPage.get(occurrence.location.page) ?? [];
    existing.push(passage);
    byPage.set(occurrence.location.page, existing);
  }
  return byPage;
}

export function useReadingChunks(
  spaceId: MaybeRefOrGetter<string | undefined>,
  node: MaybeRefOrGetter<SpaceCanvasNode | undefined>,
  kind?: MaybeRefOrGetter<SourceReaderKind | undefined>,
) {
  const currentSpaceId = computed(() => toValue(spaceId));
  const currentNode = computed(() => toValue(node));
  const currentKind = computed(() => toValue(kind));
  const needsChunks = computed(() => !currentKind.value || CHUNK_READER_KINDS.has(currentKind.value));
  const needsOcr = computed(() => !currentKind.value || OCR_READER_KINDS.has(currentKind.value));

  const chunksQuery = useQuery(
    computed(() => {
      const id = currentSpaceId.value;
      const target = currentNode.value;
      const revision = target?.data.sourceRevision ?? target?.data.knowledgeRevision ?? "";
      return {
        queryKey: ["spaces", id ?? "", "reading-chunks", target?.id ?? "", revision] as const,
        queryFn: () => (id && target ? listCurrentKnowledgeChunks(id, [target.id]) : Promise.resolve([])),
        enabled: Boolean(id && target && needsChunks.value),
      };
    }),
  );
  const recognizedText = useRecognizedText(
    currentSpaceId,
    computed(() => (needsOcr.value ? (currentNode.value?.id ?? "") : "")),
    computed(() =>
      needsOcr.value && currentNode.value?.data.kind === "file"
        ? currentNode.value.data.textRecognition?.sourceRevision
        : undefined,
    ),
  );
  const recognizedPdfPages = computed(() => {
    const target = currentNode.value;
    if (currentKind.value !== "pdf" || target?.data.kind !== "file") {
      return new Map<number, readonly string[]>();
    }
    return collectRecognizedPdfPages(target.data.documentCoverage?.occurrences ?? [], recognizedText);
  });

  return {
    chunks: computed(() => (needsChunks.value ? (chunksQuery.data.value ?? []) : [])),
    loading: computed(() => needsChunks.value && chunksQuery.isLoading.value),
    recognizedPdfPages,
    recognizedText,
  };
}
