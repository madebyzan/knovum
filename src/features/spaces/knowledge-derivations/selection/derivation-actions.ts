import type { Ref, ShallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { DocumentFormat } from "../catalog/creation-intent";
import type { CreationIntentId } from "../catalog/creation-intent-definitions";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { convertDocument } from "../document/conversion";
import { isDocumentConversionSource } from "../document/conversion-source";
import { generateGroundedDocument, type GroundedDocumentDraft } from "../grounded-document";
import { createImageCutout } from "../image-cutout";
import { splitImageLayers } from "../image-layers";
import { addVideoSubtitles } from "../subtitled-video";
import { generateVideoFromSources, type VideoDraft } from "../video/generation";
import { generateVisualization, type VisualizationDraft } from "../visualization";
import type { CreationDialogId } from "./creation-dialog";

interface DerivationActionsOptions {
  canRun(actionId: CreationIntentId): boolean;
  canSubmit(): boolean;

  selection: Ref<SpaceCanvasNode[]>;
  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly GroundedDocumentSource[] | undefined>;
  dialog: ShallowRef<CreationDialogId | undefined>;
}

export function createDerivationActions({
  canRun,
  canSubmit,
  selection,
  single,
  sources,
  dialog,
}: DerivationActionsOptions) {
  const handlers: Record<CreationIntentId, () => void> = {
    "remove-image-background": () => void (single.value && createImageCutout(single.value)),
    "split-image-layers": () => void (single.value && splitImageLayers(single.value)),
    "add-video-subtitles": () => void (single.value && addVideoSubtitles(single.value)),
    "draft-document": () => (dialog.value = "document"),
    "generate-visualization": () => (dialog.value = "visualization"),
    "convert-document-to-docx": () => void convertSelectedDocument("docx"),
    "convert-document-to-pdf": () => void convertSelectedDocument("pdf"),
    "convert-document-to-txt": () => void convertSelectedDocument("txt"),
    "convert-document-to-html": () => void convertSelectedDocument("html"),
    "render-presentation": () => void convertSelectedDocument("pptx"),
    "generate-video": () => (dialog.value = "video"),
  };

  function convertSelectedDocument(format: DocumentFormat) {
    if (!single.value || !isDocumentConversionSource(single.value)) return;
    return convertDocument({
      source: single.value,
      format,
    });
  }

  function submitDocument(draft: GroundedDocumentDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generateGroundedDocument({
      sources: sources.value,
      draft,
    });
  }

  function submitVisualization(draft: VisualizationDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generateVisualization({
      sources: sources.value,
      draft,
    });
  }

  function submitVideo(draft: VideoDraft) {
    if (!canSubmit()) return;
    const videoSources = selection.value.map((node, index) => ({
      nodeId: node.id,
      title: node.data.title,
      role: index === 0 ? ("primary" as const) : ("source" as const),
      sourceRevision: node.data.knowledgeRevision ?? "",
    }));
    if (videoSources.some((source) => !source.sourceRevision)) return;
    dialog.value = undefined;
    void generateVideoFromSources({
      sources: videoSources,
      draft,
    });
  }

  return {
    run: (actionId: CreationIntentId) => {
      if (canRun(actionId)) handlers[actionId]();
    },
    submitDocument,
    submitVisualization,
    submitVideo,
  };
}
