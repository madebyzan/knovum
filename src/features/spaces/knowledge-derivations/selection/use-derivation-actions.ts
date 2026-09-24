import type { Ref, ShallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { CreationIntentId } from "../catalog/creation-intent-catalog";
import type { GroundedDocumentSource } from "../catalog/grounded-document";
import { exportMarkdownDocument, isKnowledgeMarkdown } from "../document/export";
import { createGroundedDocument, type GroundedDocumentDraft } from "../grounded-document";
import { createImageCutout } from "../image-cutout";
import { createSubtitleResource } from "../subtitle-extraction";
import { createSubtitledVideo, type SubtitleVideoInputs } from "../subtitled-video";
import type { VideoDraft } from "../video/generation";
import { createVisualization, type VisualizationDraft } from "../visualization";
import {
  createMeetingContentPackage,
  type MeetingContentPackageDraft,
} from "../workflows/meeting-content-package";
import {
  generatePresentationFromSources,
  type PresentationGenerationDraft,
} from "../workflows/presentation-production";
import {
  createSocialContentPackage,
  type SocialContentPackageDraft,
} from "../workflows/social-content-package";
import { startSubtitledVideoWorkflow } from "../workflows/subtitled-video-production";
import { generateVideoFromSources } from "../workflows/video-production";
import type { CreationDialogId } from "./creation-dialog";

type SpaceStore = ReturnType<typeof useSpaceStore>;

interface UseDerivationActionsOptions {
  canRun(actionId: CreationIntentId): boolean;
  canSubmit(): boolean;
  store: SpaceStore;
  selection: Ref<SpaceCanvasNode[]>;
  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly GroundedDocumentSource[] | undefined>;
  subtitleInputs: Ref<SubtitleVideoInputs | undefined>;
  dialog: ShallowRef<CreationDialogId | undefined>;
}

export function useDerivationActions({
  canRun,
  canSubmit,
  store,
  selection,
  single,
  sources,
  subtitleInputs,
  dialog,
}: UseDerivationActionsOptions) {
  const handlers: Record<CreationIntentId, () => void> = {
    "remove-image-background": () => void (single.value && createImageCutout(store, single.value)),
    "extract-subtitles": () => void (single.value && createSubtitleResource(store, single.value)),
    "render-subtitled-video": () =>
      void (subtitleInputs.value && createSubtitledVideo(store, subtitleInputs.value)),
    "subtitled-video-production": () =>
      void (single.value && startSubtitledVideoWorkflow(store, single.value)),
    "meeting-content-package": () => (dialog.value = "meeting-content-package"),
    "draft-document": () => (dialog.value = "document"),
    "generate-visualization": () => (dialog.value = "visualization"),
    "export-document-as-docx": () => void exportDocument("docx"),
    "export-document-as-pdf": () => void exportDocument("pdf"),
    "presentation-production": () => (dialog.value = "presentation"),
    "video-production": () => (dialog.value = "video"),
    "social-content-package": () => (dialog.value = "social-content-package"),
  };

  function exportDocument(format: "docx" | "pdf") {
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
    void createGroundedDocument({
      store,
      selected: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitVisualization(draft: VisualizationDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void createVisualization({
      store,
      selected: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitVideo(draft: VideoDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generateVideoFromSources({
      store,
      sources: sources.value,
      draft,
    });
  }

  function submitSocial(draft: SocialContentPackageDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void createSocialContentPackage({
      store,
      sources: sources.value,
      draft,
    });
  }

  function submitMeetingPackage(draft: MeetingContentPackageDraft) {
    if (!canSubmit() || !single.value) return;
    dialog.value = undefined;
    void createMeetingContentPackage(store, single.value, draft);
  }

  function submitPresentation(draft: PresentationGenerationDraft) {
    if (!canSubmit() || !sources.value) return;
    dialog.value = undefined;
    void generatePresentationFromSources({
      store,
      sources: sources.value,
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
    submitSocial,
    submitMeetingPackage,
    submitPresentation,
  };
}
