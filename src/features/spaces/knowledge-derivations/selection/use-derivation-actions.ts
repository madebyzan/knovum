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
import type { KnowledgeVideoDraft } from "../video/knowledge-video";
import { createVisualization, type VisualizationDraft } from "../visualization";
import {
  createPresentationFromKnowledge,
  type KnowledgePresentationDraft,
} from "../workflows/knowledge-presentation";
import { createKnowledgeVideoFromKnowledge } from "../workflows/knowledge-video";
import { createMeetingVideoPackage, type MeetingVideoPackageDraft } from "../workflows/meeting-video-package";
import { createSocialContentKit, type SocialContentKitDraft } from "../workflows/social-content-kit";
import { createVideoWithSubtitles } from "../workflows/video-subtitles";
import type { CreationDialogId } from "./creation-dialog";

type SpaceStore = ReturnType<typeof useSpaceStore>;

interface UseDerivationActionsOptions {
  store: SpaceStore;
  selection: Ref<SpaceCanvasNode[]>;
  single: Ref<SpaceCanvasNode | undefined>;
  sources: Ref<readonly GroundedDocumentSource[] | undefined>;
  subtitleInputs: Ref<SubtitleVideoInputs | undefined>;
  dialog: ShallowRef<CreationDialogId | undefined>;
}

export function useDerivationActions({
  store,
  selection,
  single,
  sources,
  subtitleInputs,
  dialog,
}: UseDerivationActionsOptions) {
  const handlers: Record<CreationIntentId, () => void> = {
    "background-removal": () => void (single.value && createImageCutout(store, single.value)),
    "subtitle-extraction": () => void (single.value && createSubtitleResource(store, single.value)),
    "subtitle-burn-in": () =>
      void (subtitleInputs.value && createSubtitledVideo(store, subtitleInputs.value)),
    "video-with-subtitles": () => void (single.value && createVideoWithSubtitles(store, single.value)),
    "meeting-video-package": () => (dialog.value = "meeting-video-package"),
    "grounded-document": () => (dialog.value = "document"),
    "knowledge-visualization": () => (dialog.value = "visualization"),
    "docx-export": () => void exportDocument("docx"),
    "pdf-export": () => void exportDocument("pdf"),
    presentation: () => (dialog.value = "presentation"),
    "knowledge-video": () => (dialog.value = "video"),
    "social-content-kit": () => (dialog.value = "social-content-kit"),
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
    if (!sources.value) return;
    dialog.value = undefined;
    void createGroundedDocument({
      store,
      selected: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitVisualization(draft: VisualizationDraft) {
    if (!sources.value) return;
    dialog.value = undefined;
    void createVisualization({
      store,
      selected: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitVideo(draft: KnowledgeVideoDraft) {
    if (!sources.value) return;
    dialog.value = undefined;
    void createKnowledgeVideoFromKnowledge({
      store,
      nodes: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitSocial(draft: SocialContentKitDraft) {
    if (!sources.value) return;
    dialog.value = undefined;
    void createSocialContentKit({
      store,
      selected: selection.value,
      sources: sources.value,
      draft,
    });
  }

  function submitMeetingPackage(draft: MeetingVideoPackageDraft) {
    if (!single.value) return;
    dialog.value = undefined;
    void createMeetingVideoPackage(store, single.value, draft);
  }

  function submitPresentation(draft: KnowledgePresentationDraft) {
    if (!sources.value) return;
    dialog.value = undefined;
    void createPresentationFromKnowledge({
      store,
      nodes: selection.value,
      sources: sources.value,
      draft,
    });
  }

  return {
    run: (actionId: CreationIntentId) => handlers[actionId](),
    submitDocument,
    submitVisualization,
    submitVideo,
    submitSocial,
    submitMeetingPackage,
    submitPresentation,
  };
}
