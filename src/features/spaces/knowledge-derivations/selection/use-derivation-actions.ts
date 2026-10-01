import type { Ref, ShallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { CreationIntentId } from "../catalog/creation-intent-catalog";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { exportMarkdownDocument, isKnowledgeMarkdown, type DocumentFormat } from "../document/export";
import { generateGroundedDocument, type GroundedDocumentDraft } from "../grounded-document";
import { createImageCutout } from "../image-cutout";
import { startDerivationAction } from "../start-action";
import { addVideoSubtitles } from "../subtitled-video";
import { generateVideoFromSources, type VideoDraft } from "../video/generation";
import { generateVisualization, type VisualizationDraft } from "../visualization";
import type { CreationDialogId } from "./creation-dialog";

type SpaceStore = ReturnType<typeof useSpaceStore>;

interface UseDerivationActionsOptions {
  canRun(actionId: CreationIntentId): boolean;
  canSubmit(): boolean;
  store: SpaceStore;
  selection: Ref<SpaceCanvasNode[]>;
  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly GroundedDocumentSource[] | undefined>;
  dialog: ShallowRef<CreationDialogId | undefined>;
}

export function useDerivationActions({
  canRun,
  canSubmit,
  store,
  selection,
  single,
  sources,
  dialog,
}: UseDerivationActionsOptions) {
  const handlers: Record<CreationIntentId, () => void> = {
    "remove-image-background": () => void (single.value && createImageCutout(store, single.value)),
    "add-video-subtitles": () => void (single.value && addVideoSubtitles(store, single.value)),
    "draft-document": () => (dialog.value = "document"),
    "generate-visualization": () => (dialog.value = "visualization"),
    "export-document-as-docx": () => void exportDocument("docx"),
    "export-document-as-pdf": () => void exportDocument("pdf"),
    "export-document-as-txt": () => void exportDocument("txt"),
    "render-presentation": () => void renderPresentation(),
    "generate-video": () => (dialog.value = "video"),
  };

  function exportDocument(format: DocumentFormat) {
    if (!single.value || !isKnowledgeMarkdown(single.value)) return;
    return exportMarkdownDocument({
      store,
      source: single.value,
      format,
    });
  }

  function submitDocument(draft: GroundedDocumentDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generateGroundedDocument({
      store,
      sources: sources.value,
      draft,
    });
  }

  function submitVisualization(draft: VisualizationDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generateVisualization({
      store,
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
      store,
      sources: videoSources,
      draft,
    });
  }

  function renderPresentation() {
    if (!single.value || !sources.value) return;
    return startDerivationAction({
      store,
      sources: sources.value,
      actionId: "render-presentation",
      parameters: {},
      title: single.value.data.title,
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
