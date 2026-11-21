<script setup lang="ts">
import { computed } from "vue";

import { pdfDocumentPath } from "../../assets/file-presentation";
import type { PdfTextPage } from "../../assets/pdf-reader/pdf-editor-params";
import SpacePdfReader from "../../assets/pdf-reader/SpacePdfReader.vue";
import type { KnowledgeLocator } from "../../knowledge/model/schema/chunk";
import type { KnowledgeChunk } from "../../knowledge/model/schema/persisted-chunk";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type {
  SpaceAudioPayload,
  SpaceCodePayload,
  SpaceFilePayload,
  SpaceImagePayload,
  SpaceMarkdownPayload,
  SpaceNotePayload,
  SpaceVideoPayload,
  SpaceWebsitePayload,
} from "../../model/nodes/node-payload";
import type { SourceReaderKind } from "../source-reader-target";
import { isSpreadsheetPayload } from "../spreadsheet-sheet-projection";
import SpaceCodeReadingSurface from "./SpaceCodeReadingSurface.vue";
import SpaceDocumentReadingSurface from "./SpaceDocumentReadingSurface.vue";
import SpaceImageReadingSurface from "./SpaceImageReadingSurface.vue";
import SpaceMarkdownReadingSurface from "./SpaceMarkdownReadingSurface.vue";
import SpaceSlidesReadingSurface from "./SpaceSlidesReadingSurface.vue";
import SpaceSpreadsheetReadingSurface from "./SpaceSpreadsheetReadingSurface.vue";
import SpaceTranscriptReadingSurface from "./SpaceTranscriptReadingSurface.vue";
import SpaceWebsiteReadingSurface from "./SpaceWebsiteReadingSurface.vue";
import SpaceWordReadingSurface from "./SpaceWordReadingSurface.vue";
import type { SurfaceSelectionState } from "./use-reader-selection";

const props = defineProps<{
  node: SpaceCanvasNode;
  kind: SourceReaderKind;
  chunks: readonly KnowledgeChunk[];
  loading: boolean;
  recognizedPdfPages: ReadonlyMap<number, readonly string[]>;
  recognizedText: (resourceId: string) => string | undefined;
  locator?: KnowledgeLocator | undefined;
  quote?: string | undefined;
}>();

const emit = defineEmits<{
  pdfPageChange: [page: number];
  surfaceSelection: [state: SurfaceSelectionState | undefined];
}>();

const pdfPath = computed(() => pdfDocumentPath(props.node.data));
const pdfTargetPage = computed(() =>
  props.locator?.location.kind === "pdf" ? props.locator.location.page : undefined,
);
const isSpreadsheet = computed(() => isSpreadsheetPayload(props.node.data));

const pdfTextPages = computed<readonly PdfTextPage[]>(() => {
  if (props.kind !== "pdf") return [];
  const byPage = new Map<number, string[]>();
  for (const chunk of props.chunks) {
    if (chunk.locator.location.kind !== "pdf") continue;
    const list = byPage.get(chunk.locator.location.page) ?? [];
    list.push(chunk.text);
    byPage.set(chunk.locator.location.page, list);
  }
  return [...byPage.entries()]
    .toSorted(([left], [right]) => left - right)
    .map(([page, parts]) => ({ page, text: parts.join("\n\n") }));
});
</script>

<template>
  <SpacePdfReader
    v-if="kind === 'pdf' && pdfPath"
    :node="node"
    :path="pdfPath"
    :page="pdfTargetPage"
    :passage="quote"
    :text-pages="pdfTextPages"
    :recognized-pages="recognizedPdfPages"
    @page-change="emit('pdfPageChange', $event)"
  />
  <SpaceWordReadingSurface
    v-else-if="kind === 'word' && node.data.kind === 'file'"
    :node="node as SpaceCanvasNode<SpaceFilePayload>"
    :chunks="chunks"
    :loading="loading"
    :locator="locator"
    :quote="quote"
  />
  <SpaceSlidesReadingSurface
    v-else-if="kind === 'slides' && node.data.kind === 'file'"
    :node="node as SpaceCanvasNode<SpaceFilePayload>"
    :chunks="chunks"
    :loading="loading"
    :recognized-text="recognizedText"
    :locator="locator"
    :quote="quote"
  />
  <SpaceMarkdownReadingSurface
    v-else-if="
      (kind === 'markdown' || kind === 'note') && (node.data.kind === 'markdown' || node.data.kind === 'note')
    "
    :node="node as SpaceCanvasNode<SpaceMarkdownPayload | SpaceNotePayload>"
    :locator="locator"
    :quote="quote"
  />
  <SpaceWebsiteReadingSurface
    v-else-if="kind === 'website' && node.data.kind === 'website'"
    :node="node as SpaceCanvasNode<SpaceWebsitePayload>"
    :locator="locator"
    :quote="quote"
  />
  <SpaceTranscriptReadingSurface
    v-else-if="kind === 'media' && (node.data.kind === 'video' || node.data.kind === 'audio')"
    :node="node as SpaceCanvasNode<SpaceVideoPayload | SpaceAudioPayload>"
    :locator="locator"
    :quote="quote"
  />
  <SpaceImageReadingSurface
    v-else-if="kind === 'image' && node.data.kind === 'image'"
    :node="node as SpaceCanvasNode<SpaceImagePayload>"
    :chunks="chunks"
    :recognized-text="recognizedText"
    :quote="quote"
  />
  <SpaceCodeReadingSurface
    v-else-if="kind === 'code' && node.data.kind === 'code'"
    :node="node as SpaceCanvasNode<SpaceCodePayload>"
    :chunks="chunks"
    :locator="locator"
  />
  <SpaceSpreadsheetReadingSurface
    v-else-if="isSpreadsheet"
    :node="node"
    :chunks="chunks"
    :loading="loading"
    :locator="locator"
    :quote="quote"
    @surface-selection="emit('surfaceSelection', $event)"
  />
  <SpaceDocumentReadingSurface
    v-else
    :node="node"
    :chunks="chunks"
    :loading="loading"
    :locator="locator"
    :quote="quote"
  />
</template>
